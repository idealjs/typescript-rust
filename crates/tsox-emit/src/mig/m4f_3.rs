#![allow(unused_imports)]

use std::collections::HashMap;
use std::sync::Arc;

use tsox_core::collections::set::Set;
use tsox_core::core::compiler_options::{CompilerOptions, ScriptTarget};
use tsox_frontend::ast::node::{Node, NodeList};
use tsox_frontend::ast::node_data_generated::{
    ExpressionStatementData, NodeData, is_assignment_operator, is_computed_property_name,
    is_property_declaration,
};
use tsox_frontend::ast::subtree_facts::{
    SubtreeContainsClassFields, SubtreeContainsLexicalSuper, SubtreeContainsLexicalThis,
};
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;
use tsox_frontend::ast::visitor::NodeVisitor;
use tsox_frontend::ast::{has_syntactic_modifier, is_class_expression, ModifierFlags, NodeFlags};
use tsox_checker::binder::referenceresolver::ReferenceResolver;
use tsox_frontend::ast::mig::m3g::is_modifier;
use tsox_frontend::format::mig::m4o::EmitFlags;

use crate::mig::m3m::TransformOptions;
use crate::mig::m4g::r33k7_defs::PrivateIdentifierKind;
use crate::mig::m4h_2::{is_named_evaluation_and, transform_named_evaluation};
use crate::mig::m4k::R39K02NodeExt;
use crate::mig::m4f::r39k19_defs;
use crate::mig::m4k_2::Transformer;
use crate::printer::{EmitContext, NodeFactory};
use tsox_frontend::ast::mig::m3g_2::is_super_property;
use tsox_frontend::ast::node_data_generated::is_identifier;
use tsox_frontend::ast::node_data_generated::is_private_identifier;
use tsox_frontend::ast::utilities::has_static_modifier;
use tsox_frontend::ast::{
    is_array_literal_expression, is_assignment_expression, is_element_access_expression,
    is_left_hand_side_expression, is_object_literal_expression, is_property_access_expression,
};
use tsox_frontend::ast::node_data_generated::{
    is_property_assignment, is_shorthand_property_assignment, is_spread_assignment,
    is_spread_element,
};
use tsox_frontend::ast::mig::m3f_3::is_array_binding_or_assignment_element;
use crate::mig::w7t::is_static_property_declaration_or_class_static_block;
use crate::mig::x6a::is_anonymous_class_needing_assigned_name;
use crate::mig::m4i_8::r39k14_defs::R39K14NodeFactoryExt;
use crate::mig::m4m_5::is_simple_copiable_expression;
use crate::printer::{AutoGenerateOptions, GeneratedIdentifierFlags};

#[path = "r38k10_defs.rs"]
pub mod r38k10_defs;

#[path = "m4f_4.rs"]
pub mod m4f_4;

#[path = "m4f_5.rs"]
pub mod m4f_5;

#[path = "m4f_6.rs"]
pub mod m4f_6;

#[path = "m4f_7.rs"]
pub mod m4f_7;

use r38k10_defs::{R38K10NodeExt, R38K10NodeVisitorExt};

pub type ClassFacts = u32;

pub const CLASS_FACTS_NONE: ClassFacts = 0;
pub const CLASS_FACTS_CLASS_WAS_DECORATED: ClassFacts = 1 << 0;
pub const CLASS_FACTS_NEEDS_CLASS_CONSTRUCTOR_REFERENCE: ClassFacts = 1 << 1;
pub const CLASS_FACTS_NEEDS_CLASS_SUPER_REFERENCE: ClassFacts = 1 << 2;
pub const CLASS_FACTS_NEEDS_SUBSTITUTION_FOR_THIS_IN_CLASS_STATIC_FIELD: ClassFacts = 1 << 3;
pub const CLASS_FACTS_WILL_HOIST_INITIALIZERS_TO_CONSTRUCTOR: ClassFacts = 1 << 4;

pub struct PrivateIdentifierInfo {
    pub kind: PrivateIdentifierKind,
    pub brand_check_identifier: Option<Arc<Node>>,
    pub is_static: bool,
    pub is_valid: bool,
    pub variable_name: Option<Arc<Node>>,
    pub method_name: Option<Arc<Node>>,
    pub getter_name: Option<Arc<Node>>,
    pub setter_name: Option<Arc<Node>>,
}

pub struct PrivateEnvironmentData {
    pub class_name: Option<Arc<Node>>,
    pub weak_set_name: Option<Arc<Node>>,
}

pub struct PrivateEnvironment {
    pub data: PrivateEnvironmentData,
    pub members: HashMap<String, PrivateIdentifierInfo>,
    pub generated_identifiers: HashMap<*const Node, PrivateIdentifierInfo>,
}

#[derive(Clone)]
pub struct ClassLexicalEnvironment {
    pub facts: ClassFacts,
    pub class_constructor: Option<Arc<Node>>,
    pub class_this: Option<Arc<Node>>,
    pub super_class_reference: Option<Arc<Node>>,
}

pub struct ClassLexicalEnv {
    pub previous: Option<Box<ClassLexicalEnv>>,
    pub data: Option<ClassLexicalEnvironment>,
    pub private_env: Option<PrivateEnvironment>,
}

pub struct ClassFieldsTransformer<'a> {
    pub compiler_options: &'a CompilerOptions,
    pub resolver: Arc<dyn ReferenceResolver>,
    pub emit_context: EmitContext,

    pub should_transform_initializers_using_set: bool,
    pub should_transform_initializers_using_define: bool,
    pub should_transform_initializers: bool,
    pub should_transform_private_elements_or_class_static_blocks: bool,
    pub should_transform_auto_accessors: bool,
    pub should_transform_this_in_static_initializers: bool,
    pub should_transform_super_in_static_initializers: bool,
    pub should_transform_private_static_elements_in_file: bool,
    pub legacy_decorators: bool,

    pub pending_expressions: Vec<Arc<Node>>,
    pub pending_statements: Vec<Arc<Node>>,
    pub lexical_environment: Option<Box<ClassLexicalEnv>>,
    pub current_class_container: Option<Arc<Node>>,
    pub current_class_element: Option<Arc<Node>>,
    pub class_aliases: HashMap<*const Node, Arc<Node>>,
    pub enclosing_class_declarations: Set<*const Node>,
    pub in_iteration_statement: bool,
    pub inside_computed_property_name: bool,
    pub parent_node: Option<Arc<Node>>,
    pub current_node: Option<Arc<Node>>,

    pub modifier_visitor: Option<NodeVisitor>,
    pub discarded_value_visitor: Option<NodeVisitor>,
    pub heritage_clause_visitor: Option<NodeVisitor>,
    pub assignment_target_visitor: Option<NodeVisitor>,
    pub class_element_visitor: Option<NodeVisitor>,
    pub accessor_field_result_visitor: Option<NodeVisitor>,
    pub array_assignment_element_visitor: Option<NodeVisitor>,
    pub object_assignment_element_visitor: Option<NodeVisitor>,
    pub substitution_visitor: Option<NodeVisitor>,
}

impl ClassFieldsTransformer<'_> {
    pub fn emit_context(&self) -> &EmitContext {
        &self.emit_context
    }

    pub fn factory(&self) -> NodeFactory<'_> {
        NodeFactory::new(&self.emit_context)
    }

    pub fn visitor(&mut self) -> &mut NodeVisitor {
        self.substitution_visitor.as_mut().unwrap()
    }

    pub fn requires_block_scoped_var(&self) -> bool {
        self.in_iteration_statement
            && self.current_class_container.is_some()
            && is_class_expression(self.current_class_container.as_ref().unwrap())
    }

    pub fn new_transformer(
        &self,
        _visit: fn(&mut Self, Arc<Node>) -> Option<Arc<Node>>,
        emit_context: &EmitContext,
    ) -> Arc<Transformer> {
        Arc::new(Transformer::new(
            r39k19_defs::class_fields_transformer_visit_entry,
            Some(emit_context.clone()),
        ))
    }

    pub fn new_class_fields_transformer(opts: &TransformOptions) -> Option<Arc<Transformer>> {
        let language_version = opts.compiler_options.get_emit_script_target();
        let use_define_for_class_fields = opts.compiler_options.get_use_define_for_class_fields();

        if language_version >= ScriptTarget::ESNext && use_define_for_class_fields {
            return None;
        }

        let mut tx = Box::new(ClassFieldsTransformer {
            compiler_options: opts.compiler_options,
            resolver: opts.resolver.clone(),
            emit_context: opts.context.clone(),
            legacy_decorators: opts
                .compiler_options
                .experimental_decorators
                .is_true(),
            should_transform_initializers_using_set: false,
            should_transform_initializers_using_define: false,
            should_transform_initializers: false,
            should_transform_private_elements_or_class_static_blocks: false,
            should_transform_auto_accessors: false,
            should_transform_this_in_static_initializers: false,
            should_transform_super_in_static_initializers: false,
            should_transform_private_static_elements_in_file: false,
            pending_expressions: Vec::new(),
            pending_statements: Vec::new(),
            lexical_environment: None,
            current_class_container: None,
            current_class_element: None,
            class_aliases: HashMap::new(),
            enclosing_class_declarations: Set::new(),
            in_iteration_statement: false,
            inside_computed_property_name: false,
            parent_node: None,
            current_node: None,
            modifier_visitor: None,
            discarded_value_visitor: None,
            heritage_clause_visitor: None,
            assignment_target_visitor: None,
            class_element_visitor: None,
            accessor_field_result_visitor: None,
            array_assignment_element_visitor: None,
            object_assignment_element_visitor: None,
            substitution_visitor: None,
        });

        tx.should_transform_initializers_using_set = !use_define_for_class_fields;
        tx.should_transform_initializers_using_define =
            use_define_for_class_fields && language_version < ScriptTarget::ES2022;
        tx.should_transform_initializers = tx.should_transform_initializers_using_set
            || tx.should_transform_initializers_using_define;
        tx.should_transform_private_elements_or_class_static_blocks =
            language_version < ScriptTarget::ES2022;
        tx.should_transform_auto_accessors = language_version < ScriptTarget::ESNext;
        tx.should_transform_this_in_static_initializers = language_version < ScriptTarget::ES2022;
        tx.should_transform_super_in_static_initializers =
            tx.should_transform_this_in_static_initializers;

        let result = tx.new_transformer(|tx, node| tx.visit(&node), &opts.context);
        tx.modifier_visitor = Some(
            tx.emit_context()
                .new_node_visitor(|tx: &mut Self, node| tx.visit_modifier(&node)),
        );
        tx.discarded_value_visitor = Some(
            tx.emit_context()
                .new_node_visitor(|tx: &mut Self, node| tx.visit_discarded_value(&node)),
        );
        tx.heritage_clause_visitor = Some(
            tx.emit_context()
                .new_node_visitor(|tx: &mut Self, node| tx.visit_heritage_clause(&node)),
        );
        tx.assignment_target_visitor = Some(
            tx.emit_context()
                .new_node_visitor(|tx: &mut Self, node| tx.visit_assignment_target(&node)),
        );
        tx.class_element_visitor = Some(
            tx.emit_context()
                .new_node_visitor(|tx: &mut Self, node| tx.visit_class_element(&node)),
        );
        tx.accessor_field_result_visitor = Some(
            tx.emit_context()
                .new_node_visitor(|tx: &mut Self, node| tx.visit_accessor_field_result(&node)),
        );
        tx.array_assignment_element_visitor = Some(
            tx.emit_context()
                .new_node_visitor(|tx: &mut Self, node| tx.visit_array_assignment_element(&node)),
        );
        tx.object_assignment_element_visitor = Some(
            tx.emit_context()
                .new_node_visitor(|tx: &mut Self, node| tx.visit_object_assignment_element(&node)),
        );
        tx.substitution_visitor = Some(
            tx.emit_context()
                .new_node_visitor(|tx: &mut Self, node| tx.visit_for_substitution(&node)),
        );

        Some(result)
    }

    pub fn class_expression_needs_block_scoped_temp(&self) -> bool {
        if !self.requires_block_scoped_var() {
            return false;
        }
        if let Some(container) = &self.current_class_container {
            for member in container.members().nodes.iter() {
                if is_property_declaration(member)
                    && !has_syntactic_modifier(member, ModifierFlags::Static)
                    && member.name().is_some()
                    && is_computed_property_name(&member.name().unwrap())
                {
                    return true;
                }
            }
        }
        false
    }

    pub fn visit_source_file(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        if node.is_declaration_file_node() {
            return Some(Arc::clone(node));
        }
        self.lexical_environment = None;
        self.should_transform_private_static_elements_in_file = self
            .emit_context
            .emit_flags(node)
            .contains(EmitFlags::TRANSFORM_PRIVATE_STATIC_ELEMENTS);
        self.class_aliases = HashMap::new();
        self.enclosing_class_declarations.clear();
        let visited = self
            .visitor()
            .visit_each_child(node)
            .unwrap_or_else(|| Arc::clone(node));
        let helpers = self.emit_context.read_emit_helpers();
        self.emit_context.add_emit_helper(&visited, &helpers);
        self.class_aliases = HashMap::new();
        self.enclosing_class_declarations.clear();
        Some(visited)
    }

    pub fn visit_modifier(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        if node.kind == SyntaxKind::AccessorKeyword {
            if self.should_transform_auto_accessors_in_current_class() {
                return None;
            }
            return Some(Arc::clone(node));
        }
        if is_modifier(node) {
            return Some(Arc::clone(node));
        }
        None
    }

    pub fn should_transform_auto_accessors_in_current_class(&self) -> bool {
        if self.should_transform_auto_accessors {
            return true;
        }
        self.lexical_environment
            .as_ref()
            .and_then(|env| env.data.as_ref())
            .is_some_and(|data| {
                data.facts & CLASS_FACTS_WILL_HOIST_INITIALIZERS_TO_CONSTRUCTOR != 0
            })
    }

    pub fn visit_each_child_of_node(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        self.visitor().visit_each_child(node)
    }

    pub fn set_in_iteration_statement_and(
        &mut self,
        in_iteration: bool,
        visitor: fn(&mut Self, &Arc<Node>) -> Option<Arc<Node>>,
        node: &Arc<Node>,
    ) -> Option<Arc<Node>> {
        if self.in_iteration_statement != in_iteration {
            let saved = self.in_iteration_statement;
            self.in_iteration_statement = in_iteration;
            let result = visitor(self, node);
            self.in_iteration_statement = saved;
            return result;
        }
        visitor(self, node)
    }

    pub fn set_current_class_element_and_opt(
        &mut self,
        class_element: Option<Arc<Node>>,
        visitor: fn(&mut Self, &Arc<Node>) -> Option<Arc<Node>>,
        node: &Arc<Node>,
    ) -> Option<Arc<Node>> {
        let same_element = match (&class_element, &self.current_class_element) {
            (Some(a), Some(b)) => Arc::ptr_eq(a, b),
            (None, None) => true,
            _ => false,
        };
        if !same_element {
            let saved = self.current_class_element.take();
            self.current_class_element = class_element;
            let result = visitor(self, node);
            self.current_class_element = saved;
            return result;
        }
        visitor(self, node)
    }

    pub fn visit_function_expression_or_declaration(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        if self.current_class_element.is_some() {
            let original = self.emit_context().most_original(node);
            if !Arc::ptr_eq(&original, node) {
                if let Some(container) = &self.current_class_container {
                    for member in container.members().nodes.iter() {
                        if Arc::ptr_eq(&self.emit_context().most_original(member), &original)
                            && has_static_modifier(member)
                        {
                            return self.visit_each_child_of_node(node);
                        }
                    }
                }
            }
        }
        self.set_current_class_element_and_opt(None, Self::visit_each_child_of_node, node)
    }

    pub fn set_class_element_and_visit_each_child(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        self.set_current_class_element_and_opt(
            Some(Arc::clone(node)),
            Self::visit_each_child_of_node,
            node,
        )
    }

    pub fn push_node(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        let grandparent_node = self.parent_node.take();
        self.parent_node = self.current_node.take();
        self.current_node = Some(Arc::clone(node));
        grandparent_node
    }

    pub fn pop_node(&mut self, grandparent_node: Option<Arc<Node>>) {
        self.current_node = self.parent_node.take();
        self.parent_node = grandparent_node;
    }

    pub fn visit(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        let grandparent_node = self.push_node(node);
        let result = self.visit_with_node(node);
        self.pop_node(grandparent_node);
        result
    }

    fn visit_with_node(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        if !node
            .subtree_facts()
            .intersects(SubtreeContainsClassFields | SubtreeContainsLexicalThis | SubtreeContainsLexicalSuper)
        {
            if self.current_class_container.is_some() && !self.class_aliases.is_empty() {
                return self.visit_for_substitution(node);
            }
            return Some(Arc::clone(node));
        }

        match node.kind {
            SyntaxKind::SourceFile => self.visit_source_file(node),
            SyntaxKind::ClassDeclaration => self.visit_class_declaration(node),
            SyntaxKind::ClassExpression => self.visit_class_expression(node),
            SyntaxKind::ClassStaticBlockDeclaration | SyntaxKind::PropertyDeclaration => {
                panic!("Use `classElementVisitor` instead.")
            }
            SyntaxKind::PropertyAssignment => self.visit_property_assignment(node),
            SyntaxKind::VariableStatement => self.visit_variable_statement(node),
            SyntaxKind::VariableDeclaration => self.visit_variable_declaration(node),
            SyntaxKind::Parameter => self.visit_parameter_declaration(node),
            SyntaxKind::BindingElement => self.visit_binding_element(node),
            SyntaxKind::ExportAssignment => self.visit_export_assignment(node),
            SyntaxKind::PrivateIdentifier => self.visit_private_identifier(node),
            SyntaxKind::PropertyAccessExpression => self.visit_property_access_expression(node),
            SyntaxKind::ElementAccessExpression => self.visit_element_access_expression(node),
            SyntaxKind::PrefixUnaryExpression | SyntaxKind::PostfixUnaryExpression => {
                self.visit_pre_or_postfix_unary_expression(node, false)
            }
            SyntaxKind::BinaryExpression => self.visit_binary_expression(node, false),
            SyntaxKind::ParenthesizedExpression => {
                self.visit_parenthesized_expression(node, false)
            }
            SyntaxKind::CallExpression => self.visit_call_expression(node),
            SyntaxKind::ExpressionStatement => self.visit_expression_statement(node),
            SyntaxKind::TaggedTemplateExpression => self.visit_tagged_template_expression(node),
            SyntaxKind::ForStatement => self.visit_for_statement(node),
            SyntaxKind::ForInStatement
            | SyntaxKind::ForOfStatement
            | SyntaxKind::DoStatement
            | SyntaxKind::WhileStatement => self.set_in_iteration_statement_and(
                true,
                ClassFieldsTransformer::visit_each_child_of_node,
                node,
            ),
            SyntaxKind::ThisKeyword => self.visit_this_expression(node),
            SyntaxKind::FunctionDeclaration | SyntaxKind::FunctionExpression => self
                .set_in_iteration_statement_and(
                    false,
                    ClassFieldsTransformer::visit_function_expression_or_declaration,
                    node,
                ),
            SyntaxKind::Constructor
            | SyntaxKind::MethodDeclaration
            | SyntaxKind::GetAccessor
            | SyntaxKind::SetAccessor => self.set_in_iteration_statement_and(
                false,
                ClassFieldsTransformer::set_class_element_and_visit_each_child,
                node,
            ),
            _ => self.visitor().visit_each_child(node),
        }
    }
}

impl ClassFieldsTransformer<'_> {
    pub fn visit_for_substitution(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        if node.kind == SyntaxKind::Identifier {
            if let Some(alias) = self.class_aliases.get(&Arc::as_ptr(node)) {
                return Some(Arc::clone(alias));
            }
            return Some(Arc::clone(node));
        }
        if node.kind == SyntaxKind::PropertyAccessExpression {
            let name_is_identifier = node.name().map(|n| is_identifier(&n)).unwrap_or(false);
            if name_is_identifier {
                return self.visit_property_access_expression_for_substitution(node);
            }
        }
        self.visitor().visit_each_child(node)
    }

    fn visit_property_access_expression_for_substitution(
        &mut self,
        node: &Arc<Node>,
    ) -> Option<Arc<Node>> {
        let (expression, name) = match &node.data {
            NodeData::PropertyAccessExpression(d) => (Arc::clone(&d.expression), Arc::clone(&d.name)),
            _ => return Some(Arc::clone(node)),
        };
        let visited = self.visitor().visit_node(&expression);
        if Arc::ptr_eq(&visited, &expression) {
            return Some(Arc::clone(node));
        }
        let mut updated = self
            .factory()
            .new_property_access_expression(&visited, None, &name, node.flags);
        if let Some(u) = Arc::get_mut(&mut updated) {
            u.loc = node.loc;
        }
        Some(updated)
    }

    pub fn visit_class_declaration(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        self.visit_in_new_class_lexical_environment(
            node,
            Self::visit_class_declaration_in_new_class_lexical_environment,
        )
    }

    pub fn visit_class_expression(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        self.visit_in_new_class_lexical_environment(
            node,
            Self::visit_class_expression_in_new_class_lexical_environment,
        )
    }

    fn visit_class_declaration_in_new_class_lexical_environment(
        &mut self,
        node: &Arc<Node>,
    ) -> Option<Arc<Node>> {
        self.visit_each_child_of_node(node)
    }

    fn visit_class_expression_in_new_class_lexical_environment(
        &mut self,
        node: &Arc<Node>,
    ) -> Option<Arc<Node>> {
        self.visit_each_child_of_node(node)
    }

    fn visit_in_new_class_lexical_environment(
        &mut self,
        node: &Arc<Node>,
        visitor: fn(&mut Self, &Arc<Node>) -> Option<Arc<Node>>,
    ) -> Option<Arc<Node>> {
        let saved_current_class_container = self.current_class_container.take();
        let saved_pending_expressions = std::mem::take(&mut self.pending_expressions);
        let saved_lexical_environment = self.lexical_environment.take();
        self.current_class_container = Some(Arc::clone(node));
        self.start_class_lexical_environment();
        let original = self.emit_context.most_original(node);
        let original_ptr = Arc::as_ptr(&original);
        self.enclosing_class_declarations.add(original_ptr);
        let result = visitor(self, node);
        self.enclosing_class_declarations.delete(&original_ptr);
        self.end_class_lexical_environment();
        self.current_class_container = saved_current_class_container;
        self.pending_expressions = saved_pending_expressions;
        self.lexical_environment = saved_lexical_environment;
        result
    }

    pub fn start_class_lexical_environment(&mut self) {
        self.lexical_environment = Some(Box::new(ClassLexicalEnv {
            previous: self.lexical_environment.take(),
            data: None,
            private_env: None,
        }));
    }

    pub fn end_class_lexical_environment(&mut self) {
        self.lexical_environment = self.lexical_environment.take().and_then(|env| env.previous);
    }

    pub fn get_private_identifier_environment(&mut self) -> Option<&mut PrivateEnvironment> {
        let mut env = self.lexical_environment.as_deref_mut();
        while let Some(current) = env {
            if current.private_env.is_some() {
                return current.private_env.as_mut();
            }
            env = current.previous.as_deref_mut();
        }
        None
    }

    pub fn access_private_identifier(&mut self, name: &Arc<Node>) -> Option<PrivateIdentifierInfo> {
        if self.emit_context.has_auto_generate_info(name) {
            let key = Arc::as_ptr(name);
            let env = self.get_private_identifier_environment()?;
            return env.generated_identifiers.get(&key).cloned();
        }
        let text = name.text().to_string();
        let env = self.get_private_identifier_environment()?;
        env.members.get(&text).cloned()
    }

    pub fn visit_private_identifier(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        if !self.should_transform_private_elements_or_class_static_blocks {
            return Some(Arc::clone(node));
        }
        if self
            .parent_node
            .as_ref()
            .is_some_and(|p| r40k10_is_statement(p))
        {
            return Some(Arc::clone(node));
        }
        let result = self.factory().new_identifier("");
        self.emit_context.set_original(&result, node);
        Some(result)
    }

    pub fn visit_property_assignment(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        if is_named_evaluation_and(&self.emit_context, node, None) {
            return Some(transform_named_evaluation(&self.emit_context, node, false, ""));
        }
        self.visitor().visit_each_child(node)
    }

    pub fn visit_variable_statement(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        let saved_pending_statements = std::mem::take(&mut self.pending_statements);
        let visited_node = self
            .visitor()
            .visit_each_child(node)
            .unwrap_or_else(|| Arc::clone(node));
        if !self.pending_statements.is_empty() {
            let mut result = Vec::with_capacity(1 + self.pending_statements.len());
            result.push(visited_node);
            result.append(&mut self.pending_statements);
            self.pending_statements = saved_pending_statements;
            return Some(self.factory().new_syntax_list(result));
        }
        self.pending_statements = saved_pending_statements;
        Some(visited_node)
    }

    pub fn visit_variable_declaration(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        if is_named_evaluation_and(&self.emit_context, node, None) {
            return Some(transform_named_evaluation(&self.emit_context, node, false, ""));
        }
        self.visitor().visit_each_child(node)
    }

    pub fn visit_parameter_declaration(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        if is_named_evaluation_and(&self.emit_context, node, None) {
            return Some(transform_named_evaluation(&self.emit_context, node, false, ""));
        }
        self.visitor().visit_each_child(node)
    }

    pub fn visit_binding_element(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        if is_named_evaluation_and(&self.emit_context, node, None) {
            return Some(transform_named_evaluation(&self.emit_context, node, false, ""));
        }
        self.visitor().visit_each_child(node)
    }

    pub fn visit_export_assignment(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        if is_named_evaluation_and(&self.emit_context, node, None) {
            let is_export_equals = matches!(&node.data, NodeData::ExportAssignment(d) if d.is_export_equals);
            let assigned_name = if is_export_equals { "" } else { "default" };
            return Some(transform_named_evaluation(
                &self.emit_context,
                node,
                true,
                assigned_name,
            ));
        }
        self.visitor().visit_each_child(node)
    }

    pub fn visit_property_access_expression(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        if let Some(name) = node.name() {
            if is_private_identifier(&name) {
                if let Some(info) = self.access_private_identifier(&name) {
                    let expression = node.expression().unwrap();
                    let mut result = self.create_private_identifier_access(&info, &expression);
                    self.emit_context.set_original(&result, node);
                    if let Some(r) = Arc::get_mut(&mut result) {
                        r.loc = node.loc;
                    }
                    return Some(result);
                }
            }
        }
        let name_is_identifier = node.name().map(|n| is_identifier(&n)).unwrap_or(false);
        if name_is_identifier {
            return self.visit_property_access_expression_for_substitution(node);
        }
        self.visitor().visit_each_child(node)
    }

    pub fn visit_element_access_expression(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        self.visitor().visit_each_child(node)
    }

    pub fn visit_pre_or_postfix_unary_expression(
        &mut self,
        node: &Arc<Node>,
        _discarded: bool,
    ) -> Option<Arc<Node>> {
        self.visitor().visit_each_child(node)
    }

    pub fn visit_binary_expression(&mut self, node: &Arc<Node>, _discarded: bool) -> Option<Arc<Node>> {
        let (left, right, operator_token) = match &node.data {
            NodeData::BinaryExpression(d) => (
                Arc::clone(&d.left),
                Arc::clone(&d.right),
                Arc::clone(&d.operator_token),
            ),
            _ => return self.visitor().visit_each_child(node),
        };

        let is_destructuring_assignment = is_assignment_operator(operator_token.kind)
            && matches!(
                left.kind,
                SyntaxKind::ObjectLiteralExpression | SyntaxKind::ArrayLiteralExpression
            );
        if is_destructuring_assignment {
            let saved_pending_expressions = std::mem::take(&mut self.pending_expressions);
            let visited_left = self
                .assignment_target_visitor
                .as_mut()
                .and_then(|v| v.visit_node_opt(Some(&left)))
                .unwrap_or_else(|| Arc::clone(&left));
            let visited_right = self.visitor().visit_node(&right);
            let updated = self.factory().update_binary_expression_r39k13(
                node,
                &visited_left,
                &operator_token,
                &visited_right,
            );
            let result = if !self.pending_expressions.is_empty() {
                let mut exprs = std::mem::replace(&mut self.pending_expressions, Vec::new());
                exprs.push(updated);
                self.factory()
                    .inline_expressions(exprs)
                    .expect("expected at least one expression")
            } else {
                updated
            };
            self.pending_expressions = saved_pending_expressions;
            return Some(result);
        }

        if is_assignment_operator(operator_token.kind) {
            if is_named_evaluation_and(&self.emit_context, node, None) {
                let updated = transform_named_evaluation(&self.emit_context, node, false, "");
                return Some(updated);
            }

            let left_no_parens = r40k10_skip_partially_emitted_expressions_and_parentheses(&left);
            if left_no_parens.kind == SyntaxKind::PropertyAccessExpression {
                if let Some(name) = left_no_parens.name() {
                    if is_private_identifier(&name) {
                        if let Some(info) = self.access_private_identifier(&name) {
                            let result = self.create_private_identifier_assignment(
                                &info,
                                &left_no_parens.expression().unwrap(),
                                &right,
                                operator_token.kind,
                            );
                            let mut result = result;
                            self.emit_context.set_original(&result, node);
                            if let Some(r) = Arc::get_mut(&mut result) {
                                r.loc = node.loc;
                            }
                            return Some(result);
                        }
                    }
                }
            }
        }

        if operator_token.kind == SyntaxKind::InKeyword {
            let left = match &node.data {
                NodeData::BinaryExpression(d) => Arc::clone(&d.left),
                _ => unreachable!(),
            };
            if is_private_identifier(&left) {
                return self.transform_private_identifier_in_in_expression(node);
            }
        }

        self.visitor().visit_each_child(node)
    }

    fn transform_private_identifier_in_in_expression(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        let (left, right) = match &node.data {
            NodeData::BinaryExpression(d) => (Arc::clone(&d.left), Arc::clone(&d.right)),
            _ => return self.visitor().visit_each_child(node),
        };
        if let Some(info) = self.access_private_identifier(&left) {
            let receiver = self.visitor().visit_node(&right);
            let brand_check = info
                .brand_check_identifier
                .clone()
                .unwrap_or_else(|| Arc::clone(&right));
            let mut result = self
                .factory()
                .new_class_private_field_in_helper(&brand_check, &receiver);
            self.emit_context.set_original(&result, node);
            if let Some(r) = Arc::get_mut(&mut result) {
                r.loc = node.loc;
            }
            return Some(result);
        }
        self.visitor().visit_each_child(node)
    }

    pub fn visit_parenthesized_expression(
        &mut self,
        node: &Arc<Node>,
        discarded: bool,
    ) -> Option<Arc<Node>> {
        let expression = match &node.data {
            NodeData::ParenthesizedExpression(d) => Arc::clone(&d.expression),
            _ => return self.visitor().visit_each_child(node),
        };
        let visited = if discarded {
            self.discarded_value_visitor
                .as_mut()
                .and_then(|v| v.visit_node_opt(Some(&expression)))
        } else {
            Some(self.visitor().visit_node(&expression))
        };
        let visited = visited.unwrap_or_else(|| Arc::clone(&expression));
        Some(self.factory().update_parenthesized_expression(node, &visited))
    }

    pub fn visit_call_expression(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        let expression = node.expression().unwrap();
        if expression.kind == SyntaxKind::PropertyAccessExpression {
            if let Some(prop_name) = expression.name() {
                if is_private_identifier(&prop_name)
                    && self.access_private_identifier(&prop_name).is_some()
                {
                    let (this_arg, target) = self.create_call_binding(&expression);
                    let visited_target = self.visitor().visit_node(&target);
                    let visited_this_arg = self.visitor().visit_node(&this_arg);
                    let arguments = match &node.data {
                        NodeData::CallExpression(d) => Some(d.arguments.clone()),
                        _ => None,
                    };
                    let visited_args = self.visitor().visit_node_list(arguments.as_deref());
                    let mut all_args = vec![visited_this_arg];
                    all_args.extend(visited_args);
                    let call_target = self.factory().new_property_access_expression(
                        &visited_target,
                        None,
                        &self.factory().new_identifier("call"),
                        NodeFlags::empty(),
                    );
                    let mut result = self.factory().new_call_expression(
                        &call_target,
                        None,
                        None,
                        self.factory().new_node_list(all_args),
                        node.flags,
                    );
                    self.emit_context.set_original(&result, node);
                    if let Some(r) = Arc::get_mut(&mut result) {
                        r.loc = node.loc;
                    }
                    return Some(result);
                }
            }
        }
        self.visitor().visit_each_child(node)
    }

    pub fn visit_expression_statement(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        let expression = match &node.data {
            NodeData::ExpressionStatement(d) => Arc::clone(&d.expression),
            _ => return self.visitor().visit_each_child(node),
        };
        if is_private_identifier(&expression)
            && self.should_transform_private_elements_or_class_static_blocks
        {
            return Some(Arc::clone(node));
        }
        let visited = self
            .discarded_value_visitor
            .as_mut()
            .and_then(|v| v.visit_node_opt(Some(&expression)))
            .unwrap_or_else(|| Arc::clone(&expression));
        Some(r41k10_update_expression_statement(node, &visited))
    }

    pub fn visit_tagged_template_expression(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        self.visitor().visit_each_child(node)
    }

    pub fn visit_for_statement(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        let (initializer, condition, incrementor, statement) = match &node.data {
            NodeData::ForStatement(d) => (
                d.initializer.clone(),
                d.condition.clone(),
                d.incrementor.clone(),
                Arc::clone(&d.statement),
            ),
            _ => return self.visitor().visit_each_child(node),
        };
        let initializer = self
            .discarded_value_visitor
            .as_mut()
            .and_then(|v| v.visit_node_opt(initializer.as_ref()));
        let condition = self.visitor().visit_node_opt(condition.as_ref());
        let incrementor = self
            .discarded_value_visitor
            .as_mut()
            .and_then(|v| v.visit_node_opt(incrementor.as_ref()));
        let saved = self.in_iteration_statement;
        self.in_iteration_statement = true;
        let mut visitor = self.substitution_visitor.as_mut().unwrap();
        let body = self
            .emit_context
            .visit_iteration_body(Some(Arc::clone(&statement)), visitor);
        self.in_iteration_statement = saved;
        let body = body.unwrap_or_else(|| Arc::clone(&statement));
        Some(self.factory().update_for_statement(
            node,
            initializer.as_ref(),
            condition.as_ref(),
            incrementor.as_ref(),
            &body,
        ))
    }

    pub fn visit_this_expression(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        if self.inside_computed_property_name && self.should_transform_this_in_static_initializers {
            if let Some(data) = self.lexical_environment.as_ref().and_then(|e| e.data.as_ref()) {
                if data.facts & CLASS_FACTS_CLASS_WAS_DECORATED == 0 || self.legacy_decorators {
                    if let Some(class_this) = data.class_this.clone() {
                        return Some(class_this);
                    }
                }
            }
        }
        if self.should_transform_this_in_static_initializers && self.current_class_element.is_some()
        {
            let element = self.current_class_element.as_ref().unwrap();
            let is_static_element = element.kind == SyntaxKind::ClassStaticBlockDeclaration
                || (is_property_declaration(element) && has_static_modifier(element));
            if is_static_element {
                if let Some(data) =
                    self.lexical_environment.as_ref().and_then(|e| e.data.as_ref())
                {
                    if let Some(class_this) = data.class_this.clone() {
                        return Some(class_this);
                    }
                }
            }
        }
        Some(Arc::clone(node))
    }

    pub fn visit_discarded_value(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        match node.kind {
            SyntaxKind::PrefixUnaryExpression | SyntaxKind::PostfixUnaryExpression => {
                self.visit_pre_or_postfix_unary_expression(node, true)
            }
            SyntaxKind::BinaryExpression => self.visit_binary_expression(node, true),
            SyntaxKind::ParenthesizedExpression => self.visit_parenthesized_expression(node, true),
            _ => self.visit(node),
        }
    }

    pub fn visit_heritage_clause(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        match node.kind {
            SyntaxKind::HeritageClause => self
                .heritage_clause_visitor
                .as_mut()
                .unwrap()
                .visit_each_child(node),
            SyntaxKind::ExpressionWithTypeArguments => {
                self.visit_expression_with_type_arguments_in_heritage_clause(node)
            }
            _ => self.visit(node),
        }
    }

    fn visit_expression_with_type_arguments_in_heritage_clause(
        &mut self,
        node: &Arc<Node>,
    ) -> Option<Arc<Node>> {
        let mut facts = CLASS_FACTS_NONE;
        if let Some(data) = self.lexical_environment.as_ref().and_then(|e| e.data.as_ref()) {
            facts = data.facts;
        }
        if facts & CLASS_FACTS_NEEDS_CLASS_SUPER_REFERENCE != 0 {
            let temp = self.factory().generated_name_node(&self.factory().new_temp_variable_ex(
                AutoGenerateOptions {
                    flags: GeneratedIdentifierFlags::RESERVED_IN_NESTED_SCOPES,
                    ..Default::default()
                },
            ));
            self.emit_context.add_variable_declaration(&temp);
            self.get_class_lexical_environment().super_class_reference = Some(Arc::clone(&temp));
            let expression = match &node.data {
                NodeData::ExpressionWithTypeArguments(d) => Arc::clone(&d.expression),
                _ => unreachable!(),
            };
            let visited = self.visitor().visit_node(&expression);
            let assignment = self.factory().new_assignment_expression(&temp, &visited);
            return Some(
                self.factory()
                    .update_expression_with_type_arguments(node, assignment, None),
            );
        }
        self.heritage_clause_visitor
            .as_mut()
            .unwrap()
            .visit_each_child(node)
    }

    pub fn get_class_lexical_environment(&mut self) -> &mut ClassLexicalEnvironment {
        let env = self
            .lexical_environment
            .as_mut()
            .expect("Expected to be inside a class lexical environment.");
        if env.data.is_none() {
            env.data = Some(ClassLexicalEnvironment {
                facts: CLASS_FACTS_NONE,
                class_constructor: None,
                class_this: None,
                super_class_reference: None,
            });
        }
        env.data.as_mut().unwrap()
    }

    pub fn visit_assignment_target(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        match node.kind {
            SyntaxKind::ObjectLiteralExpression | SyntaxKind::ArrayLiteralExpression => {
                self.visit_assignment_pattern(node)
            }
            _ => self.visit(node),
        }
    }

    pub fn visit_assignment_pattern(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        if node.kind == SyntaxKind::ArrayLiteralExpression {
            let (elements, multi_line) = match &node.data {
                NodeData::ArrayLiteralExpression(d) => (d.elements.clone(), d.multi_line),
                _ => unreachable!(),
            };
            let visited = self
                .array_assignment_element_visitor
                .as_mut()
                .unwrap()
                .visit_node_list(Some(elements.as_ref()));
            return Some(
                self.factory()
                    .update_array_literal_expression(node, &visited, multi_line),
            );
        }
        let (properties, multi_line) = match &node.data {
            NodeData::ObjectLiteralExpression(d) => (d.properties.clone(), d.multi_line),
            _ => unreachable!(),
        };
        let visited = self
            .object_assignment_element_visitor
            .as_mut()
            .unwrap()
            .visit_node_list(Some(properties.as_ref()));
        Some(
            self.factory()
                .update_object_literal_expression(node, &visited, multi_line),
        )
    }

    pub fn visit_array_assignment_element(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        if is_array_binding_or_assignment_element(node) {
            if is_spread_element(node) {
                return self.visit_assignment_rest_element(node);
            }
            if node.kind != SyntaxKind::OmittedExpression {
                return self.visit_assignment_element(node);
            }
        }
        self.visitor().visit_each_child(node)
    }

    pub fn visit_assignment_rest_element(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        let spread_expression = match &node.data {
            NodeData::SpreadElement(d) => Arc::clone(&d.expression),
            _ => unreachable!(),
        };
        if is_left_hand_side_expression(&spread_expression) {
            let expression = self.visit_destructuring_assignment_target(&spread_expression)?;
            return Some(self.factory().update_spread_element(node, &expression));
        }
        self.visitor().visit_each_child(node)
    }

    pub fn visit_assignment_element(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        let mut node = Arc::clone(node);
        if is_named_evaluation_and(
            &self.emit_context,
            &node,
            Some(&is_anonymous_class_needing_assigned_name),
        ) {
            node = transform_named_evaluation(&self.emit_context, &node, false, "");
        }
        if is_assignment_expression(&node, true) {
            let (left, operator_token, right) = match &node.data {
                NodeData::BinaryExpression(d) => (
                    Arc::clone(&d.left),
                    Arc::clone(&d.operator_token),
                    Arc::clone(&d.right),
                ),
                _ => unreachable!(),
            };
            let assignment_target = self.visit_destructuring_assignment_target(&left)?;
            let initializer = self.visitor().visit_node(&right);
            return Some(self.factory().update_binary_expression_r39k13(
                &node,
                &assignment_target,
                &operator_token,
                &initializer,
            ));
        }
        self.visit_destructuring_assignment_target(&node)
    }

    pub fn visit_destructuring_assignment_target(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        if is_object_literal_expression(node) || is_array_literal_expression(node) {
            return self.visit_assignment_pattern(node);
        }
        if is_property_access_expression(node)
            && node
                .name()
                .is_some_and(|name| is_private_identifier(name))
        {
            return self.wrap_private_identifier_for_destructuring_target(node);
        }
        if self.should_transform_super_in_static_initializers
            && self.current_class_element.is_some()
            && is_super_property(node)
            && is_static_property_declaration_or_class_static_block(
                self.current_class_element.as_deref().unwrap(),
            )
        {
            let lex_data = self
                .lexical_environment
                .as_ref()
                .and_then(|lex| lex.data.clone());
            if let Some(data) = &lex_data {
                if data.facts & CLASS_FACTS_CLASS_WAS_DECORATED != 0 {
                    return self.visit_invalid_super_property(node);
                }
                if data.class_constructor.is_some() && data.super_class_reference.is_some() {
                    let mut name: Option<Arc<Node>> = None;
                    if is_element_access_expression(node) {
                        let argument = match &node.data {
                            NodeData::ElementAccessExpression(d) => {
                                Arc::clone(&d.argument_expression)
                            }
                            _ => unreachable!(),
                        };
                        name = Some(self.visitor().visit_node(&argument));
                    } else if is_property_access_expression(node) {
                        if let Some(prop_name) = node.name() {
                            if is_identifier(&prop_name) {
                                name = Some(self.factory().new_string_literal_from_node(prop_name));
                            }
                        }
                    }
                    if let Some(name) = name {
                        let temp = self
                            .factory()
                            .generated_name_node(&self.factory().new_temp_variable());
                        let super_class_reference = data.super_class_reference.clone().unwrap();
                        let class_constructor = data.class_constructor.clone().unwrap();
                        let set_expr = self.factory().new_reflect_set_call(
                            &super_class_reference,
                            &name,
                            &temp,
                            &class_constructor,
                        );
                        return Some(
                            self.factory()
                                .new_assignment_target_wrapper(&temp, &set_expr),
                        );
                    }
                }
            }
        }
        self.visitor().visit_each_child(node)
    }

    fn wrap_private_identifier_for_destructuring_target(
        &mut self,
        node: &Arc<Node>,
    ) -> Option<Arc<Node>> {
        let (expression, name) = match &node.data {
            NodeData::PropertyAccessExpression(d) => {
                (Arc::clone(&d.expression), Arc::clone(&d.name))
            }
            _ => return self.visitor().visit_each_child(node),
        };
        let parameter = self
            .factory()
            .generated_name_node(&self.factory().new_generated_name_for_node(node));
        let Some(info) = self.access_private_identifier(&name) else {
            return self.visitor().visit_each_child(node);
        };
        let mut receiver = Arc::clone(&expression);
        let is_this_or_super_property = expression.kind == SyntaxKind::ThisKeyword
            || expression.kind == SyntaxKind::SuperKeyword;
        if is_this_or_super_property || !is_simple_copiable_expression(&expression) {
            receiver = self.factory().generated_name_node(&self.factory().new_temp_variable_ex(
                AutoGenerateOptions {
                    flags: GeneratedIdentifierFlags::RESERVED_IN_NESTED_SCOPES,
                    ..Default::default()
                },
            ));
            self.emit_context.add_variable_declaration(&receiver);
            let visited = self.visitor().visit_node(&expression);
            self.pending_expressions
                .push(self.factory().new_assignment_expression(&receiver, &visited));
        }
        let assign_expr = self.create_private_identifier_assignment(
            &info,
            &receiver,
            &parameter,
            SyntaxKind::EqualsToken,
        );
        Some(self.factory().new_assignment_target_wrapper(&parameter, &assign_expr))
    }

    fn visit_invalid_super_property(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        if node.kind == SyntaxKind::PropertyAccessExpression {
            let name = node.name().unwrap();
            return Some(self.factory().update_property_access_expression(
                node,
                Some(self.factory().new_void_zero_expression()),
                None,
                Some(Arc::clone(name)),
                node.flags,
            ));
        }
        let argument = match &node.data {
            NodeData::ElementAccessExpression(d) => Arc::clone(&d.argument_expression),
            _ => unreachable!(),
        };
        let visited = self.visitor().visit_node(&argument);
        Some(self.factory().update_element_access_expression(
            node,
            Some(self.factory().new_void_zero_expression()),
            None,
            Some(visited),
            node.flags,
        ))
    }

    pub fn visit_object_assignment_element(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        if is_spread_assignment(node) {
            return self.visit_assignment_rest_property(node);
        }
        if is_shorthand_property_assignment(node) {
            return self.visit_shorthand_assignment_property(node);
        }
        if is_property_assignment(node) {
            return self.visit_assignment_property(node);
        }
        self.visitor().visit_each_child(node)
    }

    pub fn visit_assignment_rest_property(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        let spread_expression = match &node.data {
            NodeData::SpreadAssignment(d) => Arc::clone(&d.expression),
            _ => unreachable!(),
        };
        if is_left_hand_side_expression(&spread_expression) {
            let expression = self.visit_destructuring_assignment_target(&spread_expression)?;
            return Some(self.factory().update_spread_assignment(node, &expression));
        }
        self.visitor().visit_each_child(node)
    }

    pub fn visit_assignment_property(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        let (name, initializer) = match &node.data {
            NodeData::PropertyAssignment(d) => (Arc::clone(&d.name), Arc::clone(&d.initializer)),
            _ => unreachable!(),
        };
        let visited_name = self.visitor().visit_node(&name);
        if is_assignment_expression(&initializer, true) {
            if let Some(assignment_element) = self.visit_assignment_element(&initializer) {
                return Some(self.factory().update_property_assignment_r39k13(
                    node,
                    None,
                    &visited_name,
                    None,
                    None,
                    Some(&assignment_element),
                ));
            }
        } else if is_left_hand_side_expression(&initializer) {
            if let Some(target) = self.visit_destructuring_assignment_target(&initializer) {
                return Some(self.factory().update_property_assignment_r39k13(
                    node,
                    None,
                    &visited_name,
                    None,
                    None,
                    Some(&target),
                ));
            }
        }
        self.visitor().visit_each_child(node)
    }

    pub fn visit_shorthand_assignment_property(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        let mut node = Arc::clone(node);
        if is_named_evaluation_and(
            &self.emit_context,
            &node,
            Some(&is_anonymous_class_needing_assigned_name),
        ) {
            node = transform_named_evaluation(&self.emit_context, &node, false, "");
        }
        self.visitor().visit_each_child(&node)
    }
}

fn r41k10_update_expression_statement(node: &Arc<Node>, expression: &Arc<Node>) -> Arc<Node> {
    let mut updated = Node::new(
        SyntaxKind::ExpressionStatement,
        NodeData::ExpressionStatement(ExpressionStatementData {
            expression: expression.clone(),
        }),
    );
    updated.loc = node.loc;
    updated.flags = node.flags;
    Arc::new(updated)
}

fn r40k10_skip_partially_emitted_expressions_and_parentheses(node: &Arc<Node>) -> Arc<Node> {
    let mut current = Arc::clone(node);
    loop {
        match &current.data {
            NodeData::ParenthesizedExpression(d) => current = Arc::clone(&d.expression),
            NodeData::PartiallyEmittedExpression(d) => current = Arc::clone(&d.expression),
            _ => return current,
        }
    }
}

fn r40k10_is_statement(node: &Arc<Node>) -> bool {
    matches!(
        node.kind,
        SyntaxKind::VariableStatement
            | SyntaxKind::ExpressionStatement
            | SyntaxKind::IfStatement
            | SyntaxKind::DoStatement
            | SyntaxKind::WhileStatement
            | SyntaxKind::ForStatement
            | SyntaxKind::ForInStatement
            | SyntaxKind::ForOfStatement
            | SyntaxKind::ContinueStatement
            | SyntaxKind::BreakStatement
            | SyntaxKind::ReturnStatement
            | SyntaxKind::WithStatement
            | SyntaxKind::SwitchStatement
            | SyntaxKind::LabeledStatement
            | SyntaxKind::ThrowStatement
            | SyntaxKind::TryStatement
            | SyntaxKind::DebuggerStatement
            | SyntaxKind::Block
            | SyntaxKind::EmptyStatement
            | SyntaxKind::ImportDeclaration
            | SyntaxKind::ExportDeclaration
            | SyntaxKind::ExportAssignment
            | SyntaxKind::ModuleDeclaration
            | SyntaxKind::ClassDeclaration
            | SyntaxKind::FunctionDeclaration
    )
}

impl Clone for PrivateIdentifierInfo {
    fn clone(&self) -> Self {
        let kind = match self.kind {
            PrivateIdentifierKind::Field => PrivateIdentifierKind::Field,
            PrivateIdentifierKind::Method => PrivateIdentifierKind::Method,
            PrivateIdentifierKind::Accessor => PrivateIdentifierKind::Accessor,
            PrivateIdentifierKind::Untransformed => PrivateIdentifierKind::Untransformed,
        };
        PrivateIdentifierInfo {
            kind,
            brand_check_identifier: self.brand_check_identifier.clone(),
            is_static: self.is_static,
            is_valid: self.is_valid,
            variable_name: self.variable_name.clone(),
            method_name: self.method_name.clone(),
            getter_name: self.getter_name.clone(),
            setter_name: self.setter_name.clone(),
        }
    }
}
