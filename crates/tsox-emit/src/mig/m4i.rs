use std::sync::Arc;
#[allow(unused_imports)]
use tsox_core::core::compiler_options::{CompilerOptions, ModuleKind};
#[allow(unused_imports)]
use tsox_frontend::ast::node_data_generated::{
    BinaryExpressionData, BindingElementData, CatchClauseData, ExportAssignmentData,
    PropertyAssignmentData, PropertyDeclarationData, ShorthandPropertyAssignmentData,
    TaggedTemplateExpressionData, TemplateExpressionData, TemplateSpanData,
};
use tsox_frontend::ast::{
    is_no_substitution_template_literal, Node, NodeFlags, NodeList, SourceFile, SyntaxKind,
    TokenFlags,
};
#[allow(unused_imports)]
use tsox_frontend::ast::get_source_file_of_node;
#[allow(unused_imports)]
use tsox_frontend::ast::node_source_file::ScriptKind;
use tsox_frontend::ast::subtree_facts::SubtreeFacts;
use tsox_frontend::ast::mig::m3c_2::subtree_facts;
#[allow(unused_imports)]
use tsox_frontend::scanner::mig::m3i::get_source_text_of_node_from_source_file;
use tsox_frontend::scanner::{TOKEN_FLAGS_IS_INVALID, TOKEN_FLAGS_NONE};

use crate::mig::m4k_2::r37k8_defs::empty_has_file_name;
use crate::mig::m4m_5::r38k9_defs::{R38K9NodeCastExt, R38K9SourceFileNodeExt};
use crate::mig::r33k6_shim::HasFileName;
use crate::printer::{EmitContext, NodeFactory};
use crate::mig::m3m::TransformOptions;
use super::m4i_2::has_invalid_escape;
use super::m4i_11::create_not_null_condition;
use crate::mig::m4m_5::is_simple_copiable_expression;

#[path = "r39k11_defs.rs"]
pub mod r39k11_defs;

use self::r39k11_defs::R39K11SimpleTxExt;

pub struct NullishCoalescingTransformer {
    emit_context: EmitContext,
}

impl NullishCoalescingTransformer {
    fn factory(&self) -> NodeFactory<'_> {
        NodeFactory::new(&self.emit_context)
    }

    pub fn visit(&mut self, node: Arc<Node>) -> Arc<Node> {
        if !subtree_facts(&node).intersects(SubtreeFacts::CONTAINS_NULLISH_COALESCING) {
            return node;
        }
        match node.kind {
            SyntaxKind::BinaryExpression => self.visit_binary_expression(node),
            _ => self.visit_each_child(node),
        }
    }

    fn visit_binary_expression(&mut self, node: Arc<Node>) -> Arc<Node> {
        let data = node.as_binary_expression();
        match data.operator_token.kind {
            SyntaxKind::QuestionQuestionToken => {
                let mut left = self.visit_node(data.left.clone());
                let mut right = left.clone();
                if !is_simple_copiable_expression(&left) {
                    let temp = self.factory().new_temp_variable();
                    right = self.factory().generated_name_node(&temp);
                    self.emit_context.add_variable_declaration(&right);
                    left = self.factory().new_assignment_expression(&right, &left);
                }
                let when_false = self.visit_node(data.right.clone());
                let question_token = self.factory().new_token(SyntaxKind::QuestionToken);
                let colon_token = self.factory().new_token(SyntaxKind::ColonToken);
                let condition =
                    create_not_null_condition(&self.emit_context, left, right.clone(), false);
                self.factory().new_conditional_expression(
                    &condition,
                    &question_token,
                    &right,
                    &colon_token,
                    &when_false,
                )
            }
            _ => self.visit_each_child(node),
        }
    }
}

pub fn new_nullish_coalescing_transformer(opts: &TransformOptions) -> NullishCoalescingTransformer {
    NullishCoalescingTransformer {
        emit_context: opts.context.clone(),
    }
}

pub struct OptionalCatchTransformer {
    emit_context: EmitContext,
}

impl OptionalCatchTransformer {
    fn factory(&self) -> NodeFactory<'_> {
        NodeFactory::new(&self.emit_context)
    }

    pub fn visit(&mut self, node: Arc<Node>) -> Arc<Node> {
        if !subtree_facts(&node).intersects(SubtreeFacts::CONTAINS_MISSING_CATCH_CLAUSE_VARIABLE) {
            return node;
        }
        match node.kind {
            SyntaxKind::CatchClause => self.visit_catch_clause(node),
            _ => self.visit_each_child(node),
        }
    }

    fn visit_catch_clause(&mut self, node: Arc<Node>) -> Arc<Node> {
        let data = node.as_catch_clause();
        if data.variable_declaration.is_none() {
            let temp = self.factory().new_temp_variable();
            let binding = self.factory().generated_name_node(&temp);
            let variable_declaration =
                self.factory()
                    .new_variable_declaration(&binding, None, None, None);
            let block = self.visit_node(data.block.clone());
            return self
                .factory()
                .new_catch_clause(Some(&variable_declaration), &block);
        }
        self.visit_each_child(node)
    }
}

pub fn new_optional_catch_transformer(opts: &TransformOptions) -> OptionalCatchTransformer {
    OptionalCatchTransformer {
        emit_context: opts.context.clone(),
    }
}

pub struct UseStrictTransformer {
    emit_context: EmitContext,
    compiler_options: Arc<CompilerOptions>,
    get_emit_module_format_of_file: Arc<dyn Fn(&dyn HasFileName) -> ModuleKind + Send + Sync>,
}

pub fn new_use_strict_transformer(opts: &TransformOptions) -> UseStrictTransformer {
    UseStrictTransformer {
        emit_context: opts.context.clone(),
        compiler_options: Arc::new(opts.compiler_options.clone()),
        get_emit_module_format_of_file: opts.get_emit_module_format_of_file.clone(),
    }
}

impl UseStrictTransformer {
    fn factory(&self) -> NodeFactory<'_> {
        NodeFactory::new(&self.emit_context)
    }

    pub fn visit(&mut self, node: Arc<Node>) -> Arc<Node> {
        if node.kind != SyntaxKind::SourceFile {
            return node;
        }
        self.visit_source_file(node)
    }

    fn visit_source_file(&mut self, node: Arc<Node>) -> Arc<Node> {
        if node.flags.contains(NodeFlags::JsonFile) {
            return node;
        }

        let module_kind = self.compiler_options.get_emit_module_kind();
        let format = (self.get_emit_module_format_of_file)(&empty_has_file_name());

        if module_kind >= ModuleKind::ES2015
            && (module_kind == ModuleKind::Preserve || format >= ModuleKind::ES2015)
        {
            return node;
        }

        let data = node.as_source_file_data();
        let statements = self.factory().ensure_use_strict(data.statements.nodes.clone());
        let statement_list = Arc::new(NodeList {
            nodes: statements,
            loc: data.statements.loc,
        });
        self.factory().update_source_file(&node, statement_list)
    }
}

pub struct TaggedTemplateTransformer {
    emit_context: EmitContext,
    compiler_options: Arc<CompilerOptions>,
    get_emit_module_format_of_file: Arc<dyn Fn(&dyn HasFileName) -> ModuleKind + Send + Sync>,
    current_source_file: Option<Arc<Node>>,
    tagged_template_string_declarations: Vec<Arc<Node>>,
}

pub fn new_tagged_template_lift_restriction_transformer(
    opts: &TransformOptions,
) -> TaggedTemplateTransformer {
    TaggedTemplateTransformer {
        emit_context: opts.context.clone(),
        compiler_options: Arc::new(opts.compiler_options.clone()),
        get_emit_module_format_of_file: opts.get_emit_module_format_of_file.clone(),
        current_source_file: None,
        tagged_template_string_declarations: Vec::new(),
    }
}

impl TaggedTemplateTransformer {
    fn factory(&self) -> NodeFactory<'_> {
        NodeFactory::new(&self.emit_context)
    }

    pub fn visit(&mut self, node: Arc<Node>) -> Arc<Node> {
        if !subtree_facts(&node).intersects(SubtreeFacts::CONTAINS_INVALID_TEMPLATE_ESCAPE) {
            return node;
        }
        match node.kind {
            SyntaxKind::SourceFile => self.visit_source_file(node),
            SyntaxKind::TaggedTemplateExpression => self.visit_tagged_template_expression(node),
            _ => self.visit_each_child(node),
        }
    }

    fn visit_source_file(&mut self, node: Arc<Node>) -> Arc<Node> {
        self.current_source_file = Some(node.clone());
        self.tagged_template_string_declarations.clear();

        let data = node.as_source_file_data();
        let visited_statements: Vec<Arc<Node>> = data
            .statements
            .nodes
            .iter()
            .map(|s| self.visit_node(s.clone()))
            .collect();
        let mut visited = self.factory().update_source_file(
            &node,
            Arc::new(NodeList {
                nodes: visited_statements,
                loc: data.statements.loc,
            }),
        );

        if !self.tagged_template_string_declarations.is_empty() {
            let data = visited.as_source_file_data();
            let mut statements = data.statements.nodes.clone();
            let declaration_list = self.factory().new_variable_declaration_list(
                &self
                    .factory()
                    .new_node_list(self.tagged_template_string_declarations.clone()),
                NodeFlags::empty(),
            );
            statements.push(self.factory().new_variable_statement(None, &declaration_list));
            let stmt_list = Arc::new(NodeList {
                nodes: statements,
                loc: data.statements.loc,
            });
            visited = self.factory().update_source_file(&visited, stmt_list);
        }

        let helpers = self.emit_context.read_emit_helpers();
        self.emit_context.add_emit_helper(&visited, &helpers);
        visited
    }

    fn visit_tagged_template_expression(&mut self, node: Arc<Node>) -> Arc<Node> {
        self.process_tagged_template_expression(node)
    }

    fn process_tagged_template_expression(&mut self, node: Arc<Node>) -> Arc<Node> {
        let data = node.as_tagged_template_expression();
        let tag = self.visit_node(data.tag.clone());
        let template = data.template.clone();

        if !has_invalid_escape(&template) {
            return self.visit_each_child(node.clone());
        }

        let mut literals: Vec<Arc<Node>> = Vec::new();
        let mut template_arguments: Vec<Option<Arc<Node>>> = vec![None];
        if is_no_substitution_template_literal(&template) {
            literals.push(template.clone());
        } else {
            let te = template.as_template_expression();
            literals.push(te.head.clone());
            for span in &te.template_spans.nodes {
                let ts = span.as_template_span();
                literals.push(ts.literal.clone());
                template_arguments.push(Some(self.visit_node(ts.expression.clone())));
            }
        }

        let f = self.factory();
        let mut cooked_strings: Vec<Arc<Node>> = Vec::new();
        let mut raw_strings: Vec<Arc<Node>> = Vec::new();
        for literal in &literals {
            cooked_strings.push(create_template_cooked(&f, &literal.template_literal_like_data()));
            raw_strings.push(get_raw_literal(&f, literal));
        }

        let cooked_array =
            f.new_array_literal_expression(&f.new_node_list(cooked_strings), false);
        let raw_array = f.new_array_literal_expression(&f.new_node_list(raw_strings), false);
        let helper_call = f.new_template_object_helper(&cooked_array, &raw_array);

        let module_kind = self.compiler_options.get_emit_module_kind();
        let format = (self.get_emit_module_format_of_file)(&empty_has_file_name());
        let string_declaration =
            if module_kind >= ModuleKind::ES2015 && format >= ModuleKind::ES2015 {
                let temp_var = f.new_unique_name("templateObject");
                let temp_node = f.generated_name_node(&temp_var);
                let declaration = f.new_variable_declaration(&temp_node, None, None, None);
                let assigned = f.new_assignment_expression(&temp_node, &helper_call);
                template_arguments[0] = Some(f.new_logical_or_expression(&temp_node, &assigned));
                Some(declaration)
            } else {
                template_arguments[0] = Some(helper_call);
                None
            };

        let arguments = f
            .new_node_list(template_arguments.into_iter().map(Option::unwrap).collect());
        let mut call = f.new_call_expression(&tag, None, None, arguments, NodeFlags::empty());
        if let Some(call) = Arc::get_mut(&mut call) {
            call.loc = node.loc;
        }
        if let Some(declaration) = string_declaration {
            self.tagged_template_string_declarations.push(declaration);
        }
        call
    }
}

pub fn create_template_cooked(
    f: &NodeFactory<'_>,
    template: &TemplateLiteralLikeDataBase,
) -> Arc<Node> {
    if template.template_flags & TOKEN_FLAGS_IS_INVALID != 0 {
        return f.new_void_zero_expression();
    }
    f.new_string_literal(&template.text, TOKEN_FLAGS_NONE)
}

pub fn get_raw_literal(f: &NodeFactory<'_>, node: &Arc<Node>) -> Arc<Node> {
    let mut text = node.template_literal_like_data().raw_text.clone();
    if text.is_empty() {
        text = node.text().to_string();
        let is_last = node.kind == SyntaxKind::NoSubstitutionTemplateLiteral
            || node.kind == SyntaxKind::TemplateTail;
        let end_len = if is_last { 1 } else { 2 };
        text = text[1..text.len() - end_len].to_string();
    }

    text = newline_normalize(&text);

    let mut result = f.new_string_literal(&text, TOKEN_FLAGS_NONE);
    if let Some(result) = Arc::get_mut(&mut result) {
        result.loc = node.loc;
    }
    result
}

pub struct TemplateLiteralLikeDataBase {
    pub text: String,
    pub raw_text: String,
    pub template_flags: TokenFlags,
}

pub fn newline_normalize(text: &str) -> String {
    text.replace("\r\n", "\n").replace('\r', "\n")
}
