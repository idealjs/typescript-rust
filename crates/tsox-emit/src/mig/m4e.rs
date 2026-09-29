#![allow(unused_imports)]
#![allow(dead_code)]

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tsox_frontend::ast::Node;
use tsox_frontend::ast::NodeList;
use tsox_frontend::ast::ModifierList;
use tsox_frontend::ast::SyntaxKind;
use tsox_frontend::ast::node_data_generated::{
    is_binding_pattern, is_identifier, is_private_identifier, TokenFlags,
};
use tsox_frontend::ast::node_data_generated as ndg;
use tsox_frontend::ast::node_data_generated::NodeData;
use tsox_frontend::ast::node_flags::{ModifierFlags, NodeFlags};
use tsox_frontend::ast::node_source_file::LanguageVariant;
use tsox_frontend::ast::{
    get_class_extends_heritage_element, get_combined_modifier_flags, get_heritage_clause,
    get_name_of_declaration, has_static_modifier, is_big_int_literal, is_binary_expression, is_binding_element,
    is_call_expression, is_call_signature_declaration, is_class_declaration,
    is_class_static_block_declaration, is_computed_property_name, is_constructor_declaration,
    is_declaration, is_element_access_expression, is_entity_name, is_entity_name_expression,
    is_expression_statement, is_expression_with_type_arguments, is_function_declaration,
    is_function_like, is_get_accessor_declaration, is_global_scope_augmentation,
    is_heritage_clause, is_identifier_name, is_import_equals_declaration,
    is_index_signature_declaration, is_interface_declaration, is_in_js_file,
    is_js_type_alias_declaration, is_literal_expression,
    is_mapped_type_node, is_method_declaration, is_method_signature_declaration,
    is_module_declaration, is_numeric_literal, is_object_literal_expression,
    is_omitted_expression, is_parameter_declaration, is_prefix_unary_expression,
    is_property_access_expression, is_property_declaration, is_property_signature_declaration,
    is_set_accessor_declaration, is_source_file, is_source_file_js, is_string_literal,
    is_string_or_numeric_literal_like, is_syntax_list, is_type_alias_declaration,
    is_type_literal_node, is_type_parameter_declaration, is_variable_declaration,
    is_variable_declaration_list, is_external_or_common_js_module, node_is_missing,
};
use tsox_core::tspath::{get_directory_path, normalize_slashes};
use tsox_core::diagnostics::{Category, Message};
use tsox_frontend::ast::mig::m3e_4::{
    get_assignment_declaration_kind, get_element_or_property_access_name, get_this_container,
    JsDeclarationKind,
};
use tsox_frontend::ast::mig::m3f::{
    get_node_id, get_rest_indicator_of_binding_or_assignment_element,
    get_target_of_binding_or_assignment_element,
};
use tsox_frontend::ast::mig::m3f_2::has_dynamic_name;
use tsox_frontend::ast::mig::m3f_3::is_assignment_pattern;
use tsox_frontend::ast::mig::m3f_4::{is_declaration_binding_element, is_destructuring_assignment};
use tsox_frontend::ast::mig::m3g_2::is_primitive_literal_value;
use tsox_frontend::ast::mig::m3g_3::{
    is_var_await_using, is_var_using, is_variable_declaration_initialized_to_require,
    try_get_property_name_of_binding_or_assignment_element,
};
use tsox_frontend::ast::mig::w5::get_text_of_property_name;
use tsox_frontend::ast::mig::x4ast::{
    get_elements_of_binding_or_assignment_pattern,
    get_external_module_import_equals_declaration_expression,
};
use tsox_frontend::scanner::mig::m3i::is_identifier_text;
use tsox_checker::checker::mig::m1a::r19k2_defs::is_literal_import_type_node;
use tsox_frontend::ast::INTERNAL_SYMBOL_NAME_EXPORT_EQUALS;
use tsox_frontend::ast::mig::m3e_4::get_heritage_clause_element_name;
use tsox_frontend::ast::mig::m3g_2::is_this_parameter;
use tsox_frontend::ast::get_source_file_of_node;

pub use crate::printer::{AutoGenerateOptions, EmitContext, GeneratedIdentifierFlags, NodeFactory};
use crate::mig::m3n_5::r33k8_defs::FileReference;
use crate::mig::m4l_7::r38k3_defs::K3NodeAccessExt;
pub use tsox_frontend::format::mig::m4o::EmitFlags;
pub use tsox_checker::checker::mig::m2d::EmitResolver;
pub use tsox_frontend::ast::visitor::NodeVisitor as Visitor;
use crate::mig::m3m_2::create_get_symbol_accessibility_diagnostic_for_node;
use crate::mig::m3n::GetSymbolAccessibilityDiagnostic;
use crate::mig::m3n_5::create_diagnostic_for_node;
use crate::mig::m3n_7::{get_this_property_assignment_key, throw_diagnostic};
use crate::mig::m4e_2::{
    can_produce_diagnostics, get_binding_name_visible, is_declaration_and_not_visible,
    is_enclosing_declaration, is_private_method_type_parameter, is_scope_marker,
    should_emit_function_properties, unwrap_parenthesized_expression,
};
use tsox_frontend::ast::mig::w5::get_leftmost_access_expression;
use crate::mig::m4m_2::{is_original_node_single_line, is_simple_inlineable_expression};

#[path = "r37k1_defs.rs"]
pub mod r37k1_defs;
pub use r37k1_defs::*;

#[path = "r38k1_defs.rs"]
pub mod r38k1_defs;
pub use r38k1_defs::*;

#[path = "r39k01_defs.rs"]
pub mod r39k01_defs;
pub use r39k01_defs::*;

pub trait DeclarationEmitHost {
    fn get_effective_declaration_flags(&self, node: &Arc<Node>, flags: ModifierFlags) -> ModifierFlags;
}

pub struct DeclarationTransformerState {
    pub current_source_file: Option<Arc<Node>>,
    pub file: Option<Arc<tsox_frontend::ast::SourceFile>>,
    pub isolated_declarations: bool,
    pub strip_internal: bool,
    pub late_marked_statements: Vec<Arc<Node>>,
    pub raw_referenced_files: Vec<ReferencedFilePair>,
    pub raw_type_reference_directives: Vec<FileReference>,
    pub raw_lib_reference_directives: Vec<FileReference>,
    pub get_symbol_accessibility_diagnostic: Option<GetSymbolAccessibilityDiagnostic>,
    pub error_name_node: Option<Arc<Node>>,
    pub diagnostics: Vec<tsox_frontend::ast::diagnostic::Diagnostic>,
}

impl Default for DeclarationTransformerState {
    fn default() -> Self {
        Self {
            current_source_file: None,
            file: None,
            isolated_declarations: false,
            strip_internal: false,
            late_marked_statements: Vec::new(),
            raw_referenced_files: Vec::new(),
            raw_type_reference_directives: Vec::new(),
            raw_lib_reference_directives: Vec::new(),
            get_symbol_accessibility_diagnostic: None,
            error_name_node: None,
            diagnostics: Vec::new(),
        }
    }
}

impl DeclarationTransformerState {
    pub fn add_diagnostic(&mut self, diagnostic: tsox_frontend::ast::diagnostic::Diagnostic) {
        self.diagnostics.push(diagnostic);
    }
}

pub trait EmitTracker {
    fn report_inference_fallback(&self, node: &Arc<Node>);
}

pub mod Diagnostics {
    use tsox_core::diagnostics::{Category, Message};
    pub use tsox_core::diagnostics::messages_generated::{
        COMPUTED_PROPERTY_NAMES_ON_CLASS_OR_OBJECT_LITERALS_CANNOT_BE_INFERRED_WITH_ISOLATEDDECLARATIONS as ComputedPropertyNamesOnClassOrObjectLiteralsCannotBeInferredWithIsolatedDeclarations,
        COMPUTED_PROPERTIES_MUST_BE_NUMBER_OR_STRING_LITERALS_VARIABLES_OR_DOTTED_EXPRESSIONS_WITH_ISOLATEDDECLARATIONS as ComputedPropertiesMustBeNumberOrStringLiteralsVariablesOrDottedExpressionsWithIsolatedDeclarations,
        DECLARATION_EMIT_FOR_THIS_FILE_REQUIRES_PRESERVING_THIS_IMPORT_FOR_AUGMENTATIONS_THIS_IS_NOT_SUPPORTED_WITH_ISOLATEDDECLARATIONS as DeclarationEmitForThisFileRequiresPreservingThisImportForAugmentationsThisIsNotSupportedWithIsolatedDeclarations,
        MULTIPLE_MODULE_EXPORTS_ASSIGNMENTS_CANNOT_BE_SERIALIZED_FOR_DECLARATION_EMIT as MultipleModuleExportsAssignmentsCannotBeSerializedForDeclarationEmit,
    };

    pub static DeclarationEmitElidesPrivateMembersBut0RefersToAPrivateMemberWriteAnExplicitTypeHere: Message = Message {
        code: 7080,
        category: Category::Error,
        key: "Declaration_emit_elides_private_members_but_0_refers_to_a_private_member_Write_an_explicit_type_here_7080",
        text: "Declaration emit elides private members, but '{0}' refers to a private member. Write an explicit type here.",
        reports_unnecessary: false,
        elided_in_compatibility_pyramid: false,
        reports_deprecated: false,
    };
}

pub struct DeclarationTransformer {
    pub enclosing_declaration: Option<Arc<Node>>,
    pub needs_declare: bool,
    pub needs_scope_fix_marker: bool,
    pub result_has_scope_marker: bool,
    pub result_has_external_module_indicator: bool,
    pub suppress_new_diagnostic_contexts: bool,
    pub cjs_export_assignment: Option<Arc<Node>>,
    pub cjs_export_assignment_name: Option<Arc<Node>>,
    pub cjs_export_members: Vec<Arc<Node>>,
    pub witnessed_cjs_exports: HashSet<String>,
    pub late_statement_replacement_map: HashMap<u64, Option<Arc<Node>>>,
    pub expando_hosts: HashMap<u64, Arc<Node>>,
    pub expando_members: HashMap<u64, Vec<Arc<Node>>>,
    pub deferred_expando_assignments: HashMap<u64, Vec<Arc<Node>>>,
    pub seen_properties: HashSet<(u64, bool, String)>,
    pub this_property_assignments_collected: Vec<Arc<Node>>,
    pub declaration_file_path: String,
    pub emit_context: EmitContext,
    pub resolver: EmitResolver,
    pub host: Box<dyn DeclarationEmitHost>,
    pub tracker: Box<dyn EmitTracker>,
    pub state_data: DeclarationTransformerState,
}

impl DeclarationTransformer {
    fn factory(&self) -> NodeFactory<'_> {
        NodeFactory::new(&self.emit_context)
    }

    fn visitor(&self) -> Visitor {
        Visitor::default()
    }

    fn emit_context(&self) -> &EmitContext {
        &self.emit_context
    }

    fn resolver(&self) -> &EmitResolver {
        &self.resolver
    }

    fn host(&self) -> &dyn DeclarationEmitHost {
        self.host.as_ref()
    }

    fn state(&mut self) -> &mut DeclarationTransformerState {
        &mut self.state_data
    }

    fn tracker(&self) -> &dyn EmitTracker {
        self.tracker.as_ref()
    }

    pub fn visit(&mut self, node: Option<Arc<Node>>) -> Option<Arc<Node>> {
        let node = node?;
        match node.kind {
            SyntaxKind::SourceFile => self.visit_source_file(node),
            SyntaxKind::FunctionDeclaration
            | SyntaxKind::ModuleDeclaration
            | SyntaxKind::ImportEqualsDeclaration
            | SyntaxKind::InterfaceDeclaration
            | SyntaxKind::ClassDeclaration
            | SyntaxKind::JSTypeAliasDeclaration
            | SyntaxKind::TypeAliasDeclaration
            | SyntaxKind::EnumDeclaration
            | SyntaxKind::VariableStatement
            | SyntaxKind::ImportDeclaration
            | SyntaxKind::JSImportDeclaration
            | SyntaxKind::ExportDeclaration
            | SyntaxKind::ExportAssignment => self.visit_declaration_statements(node),
            SyntaxKind::BreakStatement
            | SyntaxKind::ContinueStatement
            | SyntaxKind::DebuggerStatement
            | SyntaxKind::DoStatement
            | SyntaxKind::EmptyStatement
            | SyntaxKind::ForInStatement
            | SyntaxKind::ForOfStatement
            | SyntaxKind::ForStatement
            | SyntaxKind::IfStatement
            | SyntaxKind::LabeledStatement
            | SyntaxKind::ReturnStatement
            | SyntaxKind::SwitchStatement
            | SyntaxKind::ThrowStatement
            | SyntaxKind::TryStatement
            | SyntaxKind::WhileStatement
            | SyntaxKind::WithStatement
            | SyntaxKind::NotEmittedStatement
            | SyntaxKind::Block
            | SyntaxKind::MissingDeclaration
            | SyntaxKind::ExpressionStatement => None,
            _ => self.visit_declaration_subtree(node),
        }
    }

    pub fn visit_source_file(&mut self, node: Arc<Node>) -> Option<Arc<Node>> {
        self.cjs_export_assignment_name = None;
        if self
            .state_data
            .file
            .as_ref()
            .map(|f| f.is_declaration_file)
            .unwrap_or(false)
        {
            return Some(node);
        }

        self.needs_declare = true;
        self.needs_scope_fix_marker = false;
        self.result_has_scope_marker = false;
        self.enclosing_declaration = Some(node.clone());
        self.state().get_symbol_accessibility_diagnostic = Some(Box::new(throw_diagnostic));
        self.result_has_external_module_indicator = false;
        self.suppress_new_diagnostic_contexts = false;
        self.state().late_marked_statements = Vec::new();
        self.late_statement_replacement_map = HashMap::new();
        self.expando_hosts = HashMap::new();
        self.expando_members = HashMap::new();
        self.deferred_expando_assignments = HashMap::new();
        self.state().current_source_file = Some(node.clone());
        self.collect_file_references();
        if let Some(file) = &self.state_data.file {
            self.resolver
                .precalculate_declaration_emit_visibility(file);
        }
        let updated = self.transform_source_file(node);
        self.state().current_source_file = None;
        updated
    }

    fn collect_file_references(&mut self) {
        let Some(file) = self.state_data.file.clone() else {
            return;
        };
        let convert = |r: &tsox_frontend::ast::node_source_file::FileReference| FileReference {
            file_name: r.file_name.clone(),
            text_range: r.range,
            resolution_mode: Some(r.resolution_mode),
            preserve: r.preserve,
        };
        self.state().raw_referenced_files = file
            .referenced_files
            .iter()
            .map(|r| ReferencedFilePair {
                file: file.node.clone(),
                r#ref: convert(r),
            })
            .collect();
        self.state().raw_type_reference_directives =
            file.type_reference_directives.iter().map(convert).collect();
        self.state().raw_lib_reference_directives =
            file.lib_reference_directives.iter().map(convert).collect();
    }

    fn append_cjs_exports(&mut self, combined_statements: Arc<NodeList>) -> Arc<NodeList> {
        let mut result: Vec<Arc<Node>> = Vec::new();
        if let Some(cjs) = &self.cjs_export_assignment {
            result.push(cjs.clone());
        }
        result.extend(self.cjs_export_members.iter().cloned());
        result.extend(combined_statements.nodes.iter().cloned());
        let statement_nodes = flatten_syntax_lists(&result);
        if statement_nodes.len() != combined_statements.nodes.len() {
            return self.factory().new_node_list(statement_nodes);
        }
        combined_statements
    }

    pub fn transform_source_file(&mut self, node: Arc<Node>) -> Option<Arc<Node>> {
        self.cjs_export_assignment = None;
        self.cjs_export_assignment_name = None;
        self.cjs_export_members = Vec::new();
        self.witnessed_cjs_exports = HashSet::new();
        let result = self.transform_source_file_worker(node.clone());
        self.cjs_export_assignment = None;
        self.cjs_export_assignment_name = None;
        self.cjs_export_members = Vec::new();
        self.witnessed_cjs_exports = HashSet::new();
        result
    }

    fn transform_source_file_worker(&mut self, node: Arc<Node>) -> Option<Arc<Node>> {
        self.cjs_export_assignment_visitor().visit_node(&node);
        self.expression_visitor().visit_node(&node);
        let statements = self.visitor().visit_nodes(node.as_source_file().statements.clone());
        let mut combined_statements = self.transform_and_replace_late_painted_statements(statements);
        combined_statements = self.append_cjs_exports(combined_statements);
        combined_statements = Arc::new(NodeList {
            loc: node.as_source_file().statements.loc,
            nodes: combined_statements.nodes.clone(),
        });
        if self
            .state_data
            .file
            .as_ref()
            .map(|f| is_external_or_common_js_module(f))
            .unwrap_or(false)
        {
            // Go 349-360: JS 文件 file.Symbol.Exports[ExportEquals] 多 export= 诊断,
            // 依赖 Node→Symbol 接线(m3a symbol() 属 Type,K3NodeAccessExt 未接 symbol map),留 r43 交接
            if !self.result_has_external_module_indicator
                || (self.needs_scope_fix_marker && !self.result_has_scope_marker)
            {
                let marker = create_empty_exports(&self.factory());
                let mut new_list = combined_statements.nodes.clone();
                new_list.push(marker);
                combined_statements = Arc::new(NodeList {
                    loc: combined_statements.loc,
                    nodes: new_list,
                });
            }
        }
        let result = self.factory().update_source_file(&node, combined_statements);
        Some(result)
    }

    pub fn visit_declaration_subtree(&mut self, input: Arc<Node>) -> Option<Arc<Node>> {
        if self.should_strip_internal(Some(&input)) {
            return None;
        }
        if is_declaration(&input)
            && is_declaration_and_not_visible(&self.emit_context(), &self.resolver(), &input)
        {
            return None;
        }
        if has_dynamic_name(&input) {
            if self.state().isolated_declarations {
                if !self
                    .resolver()
                    .is_definitely_reference_to_global_symbol_object(input.name().unwrap().expression().unwrap())
                {
                    let parent_is_class_or_object = input
                        .parent()
                        .as_deref()
                        .map(|p| is_class_declaration(p) || is_object_literal_expression(p))
                        .unwrap_or(false);
                    if parent_is_class_or_object
                    {
                        self.state().add_diagnostic(create_diagnostic_for_node(
                            &input,
                            Some(&Diagnostics::ComputedPropertyNamesOnClassOrObjectLiteralsCannotBeInferredWithIsolatedDeclarations),
                            &[],
                        ));
                        return None;
                    } else if input
                        .parent()
                        .as_deref()
                        .map(|p| is_interface_declaration(p) || is_type_literal_node(p))
                        .unwrap_or(false)
                        && !is_entity_name_expression(input.name().unwrap().expression().unwrap())
                    {
                        self.state().add_diagnostic(create_diagnostic_for_node(
                            &input,
                            Some(&Diagnostics::ComputedPropertiesMustBeNumberOrStringLiteralsVariablesOrDottedExpressionsWithIsolatedDeclarations),
                            &[],
                        ));
                        return None;
                    }
                }
            } else if !self.resolver.is_late_bound(self.emit_context().parse_node(&input).as_ref())
                || !is_entity_name_expression(input.name().unwrap().expression().unwrap())
            {
                return None;
            }
        }

        if is_function_like(&input) && self.resolver.is_implementation_of_overload(&input) {
            return None;
        }

        if input.kind == SyntaxKind::SemicolonClassElement {
            return None;
        }

        if is_heritage_clause(&input)
            && (input.as_heritage_clause().types.nodes.is_empty()
                || (input.as_heritage_clause().types.nodes.len() == 1
                    && node_is_missing(Some(&input.as_heritage_clause().types.nodes[0]))))
        {
            return None;
        }

        let previous_enclosing_declaration = self.enclosing_declaration.clone();
        if is_enclosing_declaration(&input) {
            self.enclosing_declaration = Some(input.clone());
        }

        let (can_produce_diagnostic, cleanup_diagnostic_context) =
            self.setup_diagnostic_context(&input);

        let result = match input.kind {
            SyntaxKind::MappedType => self.transform_mapped_type_node(&input),
            SyntaxKind::HeritageClause => self.transform_heritage_clause(&input),
            SyntaxKind::MethodSignature => self.transform_method_signature_declaration(&input),
            SyntaxKind::MethodDeclaration => self.transform_method_declaration(&input),
            SyntaxKind::ConstructSignature => self.transform_construct_signature_declaration(&input),
            SyntaxKind::Constructor => self.transform_constructor_declaration(&input),
            SyntaxKind::GetAccessor => self.transform_get_accessor_declaration(&input),
            SyntaxKind::SetAccessor => self.transform_set_accessor_declaration(&input),
            SyntaxKind::PropertyDeclaration => self.transform_property_declaration(&input),
            SyntaxKind::PropertySignature => self.transform_property_signature_declaration(&input),
            SyntaxKind::CallSignature => self.transform_call_signature_declaration(&input),
            SyntaxKind::IndexSignature => self.transform_index_signature_declaration(&input),
            SyntaxKind::VariableDeclaration => self.transform_variable_declaration(&input),
            SyntaxKind::TypeParameter => self.transform_type_parameter_declaration(&input),
            SyntaxKind::ExpressionWithTypeArguments => self.transform_expression_with_type_arguments(&input),
            SyntaxKind::TypeReference => self.transform_type_reference(&input),
            SyntaxKind::ConditionalType => self.transform_conditional_type_node(&input),
            SyntaxKind::FunctionType => self.transform_function_type_node(&input),
            SyntaxKind::ConstructorType => self.transform_constructor_type_node(&input),
            SyntaxKind::ImportType => self.transform_import_type_node(&input),
            SyntaxKind::TypeQuery => {
                let expr_name = input.as_type_query_node().expr_name.clone();
                let enclosing = self.enclosing_declaration.clone();
                self.check_entity_name_visibility(&expr_name, enclosing.as_ref());
                self.visitor().visit_each_child(&input)
            }
            SyntaxKind::QualifiedName => {
                let right = input.as_qualified_name().right.clone();
                if right.kind == SyntaxKind::PrivateIdentifier {
                    self.state().add_diagnostic(create_diagnostic_for_node(
                        &input,
                        Some(&Diagnostics::DeclarationEmitElidesPrivateMembersBut0RefersToAPrivateMemberWriteAnExplicitTypeHere),
                        &[right.text().to_string()],
                    ));
                }
                self.visitor().visit_each_child(&input)
            }
            SyntaxKind::TupleType => {
                let result = self.visitor().visit_each_child(&input);
                if let Some(result) = &result {
                    if is_original_node_single_line(&self.emit_context(), Some(&input)) {
                        self.emit_context().add_emit_flags(result, EmitFlags::SINGLE_LINE);
                    }
                }
                result
            }
            SyntaxKind::JSDocTypeExpression => self.transform_jsdoc_type_expression(&input),
            SyntaxKind::JSDocTypeLiteral => self.transform_jsdoc_type_literal(&input),
            SyntaxKind::JSDocPropertyTag => self.transform_jsdoc_property_tag(&input),
            SyntaxKind::JSDocAllType => self.transform_jsdoc_all_type(&input),
            SyntaxKind::JSDocNullableType => self.transform_jsdoc_nullable_type(&input),
            SyntaxKind::JSDocNonNullableType => self.transform_jsdoc_non_nullable_type(&input),
            SyntaxKind::JSDocOptionalType => self.transform_jsdoc_optional_type(&input),
            SyntaxKind::JSDocVariadicType => self.transform_jsdoc_variadic_type(&input),
            _ => self.visitor().visit_each_child(&input),
        };

        if let Some(_) = &result {
            if can_produce_diagnostic && has_dynamic_name(&input) {
                self.check_name(&input);
            }
        }

        cleanup_diagnostic_context(self);
        self.enclosing_declaration = previous_enclosing_declaration;
        result
    }

    pub fn transform_mapped_type_node(&mut self, input: &Arc<Node>) -> Option<Arc<Node>> {
        let mt = input.as_mapped_type_node();
        let type_node = match &mt.type_node {
            None => Some(self.factory().new_keyword_type_node(SyntaxKind::AnyKeyword)),
            Some(t) => self.visitor().visit(t.clone()),
        };
        Some(self.factory().update_mapped_type_node(
            input,
            mt.readonly_token.clone(),
            self.visitor().visit_opt(Some(mt.type_parameter.clone())),
            self.visitor().visit_opt(mt.name_type.clone()),
            mt.question_token.clone(),
            type_node,
            None,
        ))
    }

    pub fn transform_heritage_clause(&mut self, clause: &Arc<Node>) -> Option<Arc<Node>> {
        let hc = clause.as_heritage_clause();
        let retained_clauses: Vec<Arc<Node>> = hc
            .types
            .nodes
            .iter()
            .filter(|t| {
                let name = get_heritage_clause_element_name(t);
                name.as_ref().map(|name| is_entity_name(name) || is_entity_name_expression(name))
                    .unwrap_or(false)
                    || (hc.token == SyntaxKind::ExtendsKeyword
                        && is_expression_with_type_arguments(t)
                        && t.expression().is_some_and(|e| e.kind == SyntaxKind::NullKeyword))
            })
            .cloned()
            .collect();
        if retained_clauses.is_empty() {
            return None;
        }
        if retained_clauses.len() == hc.types.nodes.len() {
            return self.visitor().visit_each_child(clause);
        }
        Some(self.factory().update_heritage_clause(
            clause,
            hc.token,
            &self
                .visitor()
                .visit_nodes(self.factory().new_node_list(retained_clauses)),
        ))
    }

    pub fn transform_import_type_node(&mut self, input: &Arc<Node>) -> Option<Arc<Node>> {
        if !is_literal_import_type_node(input) {
            return Some(input.clone());
        }
        let it = input.as_import_type_node();
        let argument = it.argument.as_literal_type_node();
        let rewritten = self.rewrite_module_specifier(input, Some(&argument.literal));
        let rewritten = rewritten.unwrap_or_else(|| argument.literal.clone());
        Some(self.factory().update_import_type_node(
            input,
            it.is_type_of,
            self.factory().update_literal_type_node(&it.argument, rewritten),
            it.attributes.clone(),
            it.qualifier.clone(),
            it.type_arguments.clone().map(|ta| self.visitor().visit_nodes(ta)),
        ))
    }

    pub fn transform_type_reference(&mut self, input: &Arc<Node>) -> Option<Arc<Node>> {
        let type_name = input.as_type_reference_node().type_name.clone();
        let enclosing = self.enclosing_declaration.clone();
        self.check_entity_name_visibility(&type_name, enclosing.as_ref());
        self.visitor().visit_each_child(input)
    }

    pub fn transform_expression_with_type_arguments(&mut self, input: &Arc<Node>) -> Option<Arc<Node>> {
        let expression = input.as_expression_with_type_arguments().expression.clone();
        if is_entity_name(&expression) || is_entity_name_expression(&expression) {
            let enclosing = self.enclosing_declaration.clone();
            self.check_entity_name_visibility(&expression, enclosing.as_ref());
        }
        self.visitor().visit_each_child(input)
    }

    pub fn transform_type_parameter_declaration(&mut self, input: &Arc<Node>) -> Option<Arc<Node>> {
        if is_private_method_type_parameter(self.host(), input)
            && (input.as_type_parameter_declaration().default_type.is_some()
                || input.as_type_parameter_declaration().constraint.is_some())
        {
            let tp = input.as_type_parameter_declaration();
            return Some(self.factory().update_type_parameter_declaration(
                input,
                input.modifiers().cloned(),
                input.name().cloned(),
                None,
                tp.expression.clone(),
                None,
            ));
        }
        self.visitor().visit_each_child(input)
    }

    pub fn transform_variable_declaration(&mut self, input: &Arc<Node>) -> Option<Arc<Node>> {
        if self.state_data.file.as_ref().map(|f| f.common_js_module_indicator.is_some()).unwrap_or(false)
            && is_variable_declaration_initialized_to_require(input)
        {
            return self.transform_cjs_require_variable_declaration(input);
        }
        let Some(name) = input.name() else {
            return self.visitor().visit_each_child(input);
        };
        if is_binding_pattern(name) && has_any_binding_initializers(name) {
            return self.recreate_binding_pattern(name);
        }
        self.suppress_new_diagnostic_contexts = true;
        let visited_name = self.binding_name_visitor().visit_node(name);
        let ensured_type = self.ensure_type(input, false);
        let no_initializer = self.ensure_no_initializer(input);
        Some(self.factory().update_variable_declaration_node(
            input,
            &visited_name,
            None,
            ensured_type,
            no_initializer,
        ))
    }

    fn transform_cjs_require_variable_declaration(&mut self, input: &Arc<Node>) -> Option<Arc<Node>> {
        let initializer_args = &input.initializer().unwrap().as_call_expression().arguments.nodes;
        let specifier = self.rewrite_module_specifier(input, Some(&initializer_args[0]));
        let Some(specifier) = specifier else {
            return None;
        };
        let Some(name) = input.name().cloned() else {
            return None;
        };
        if is_identifier(&name) {
            Some(self.factory().new_import_equals_declaration(
                None,
                false,
                name,
                self.factory().new_external_module_reference(specifier),
            ))
        } else if name.kind == SyntaxKind::ArrayBindingPattern {
            None
        } else {
            let b = name.as_binding_pattern();
            let mut import_specifiers: Vec<Arc<Node>> = Vec::new();
            for elem in &b.elements.nodes {
                if !elem.name().map(|n| is_identifier(n)).unwrap_or(false) {
                    continue;
                }
                import_specifiers.push(self.factory().new_import_specifier(
                    false,
                    elem.property_name().cloned(),
                    elem.name().cloned().expect("import specifier name"),
                ));
            }
            Some(self.factory().new_import_declaration(
                None,
                Some(self.factory().new_import_clause(
                    SyntaxKind::Unknown,
                    None,
                    Some(self.factory()
                        .new_named_imports(self.factory().new_node_list(import_specifiers))),
                )),
                specifier,
                None,
            ))
        }
    }

    fn recreate_binding_pattern(&mut self, input: &Arc<Node>) -> Option<Arc<Node>> {
        let mut results: Vec<Arc<Node>> = Vec::new();
        for elem in &input.as_binding_pattern().elements.nodes {
            let result = self.recreate_binding_element(elem)?;
            if result.kind == SyntaxKind::SyntaxList {
                results.extend(result.as_syntax_list().children.iter().cloned());
            } else {
                results.push(result);
            }
        }
        if results.is_empty() {
            return None;
        }
        if results.len() == 1 {
            return Some(results.into_iter().next().unwrap());
        }
        Some(self.factory().new_syntax_list(results))
    }

    fn recreate_binding_element(&mut self, e: &Arc<Node>) -> Option<Arc<Node>> {
        let name = e.name()?;
        if !get_binding_name_visible(&self.resolver(), e) {
            return None;
        }
        if is_binding_pattern(&name) {
            return self.recreate_binding_pattern(&name);
        }
        let ensured_type = self.ensure_type(e, false);
        Some(self.factory().new_variable_declaration(
            name,
            None,
            ensured_type.as_ref(),
            None,
        ))
    }

    pub fn transform_index_signature_declaration(&mut self, input: &Arc<Node>) -> Option<Arc<Node>> {
        let mut t = self.visitor().visit(input.as_index_signature_declaration().type_node.clone());
        if t.is_none() {
            t = Some(self.factory().new_keyword_type_node(SyntaxKind::AnyKeyword));
        }
        let params = input.as_index_signature_declaration().parameters.clone();
        let mods = self.ensure_modifiers(input);
        let new_params = self.update_param_list(input, &params);
        Some(self.factory().update_index_signature_declaration(
            input,
            mods,
            new_params,
            t,
        ))
    }

    pub fn transform_property_signature_declaration(&mut self, input: &Arc<Node>) -> Option<Arc<Node>> {
        if input.name().map(|n| is_private_identifier(n)).unwrap_or(false) {
            return None;
        }
        let ps = input.as_property_signature_declaration();
        let mods = self.ensure_modifiers(input);
        let ensured_type = self.ensure_type(input, false);
        let no_initializer = self.ensure_no_initializer(input);
        let result = self.factory().update_property_signature_declaration(
            input,
            mods,
            input.name().unwrap().clone(),
            ps.postfix_token.clone(),
            ensured_type,
            no_initializer,
        );
        self.preserve_partial_js_doc(&result, input);
        Some(result)
    }

    pub fn transform_property_declaration(&mut self, input: &Arc<Node>) -> Option<Arc<Node>> {
        if input.name().map(|n| is_private_identifier(n)).unwrap_or(false) {
            return None;
        }
        let mut postfix_token = input.as_property_declaration().postfix_token.clone();
        if let Some(tok) = &postfix_token {
            if tok.kind == SyntaxKind::ExclamationToken {
                postfix_token = None;
            }
        }
        let ensured_type = self.ensure_type(input, false);
        let no_initializer = self.ensure_no_initializer(input);
        Some(self.factory().update_property_declaration(
            input,
            self.ensure_modifiers(input),
            input.name().unwrap(),
            postfix_token,
            ensured_type,
            no_initializer,
        ))
    }

    pub fn transform_set_accessor_declaration(&mut self, input: &Arc<Node>) -> Option<Arc<Node>> {
        if input.name().map(|n| is_private_identifier(n)).unwrap_or(false) {
            return None;
        }
        let params = input.as_set_accessor_declaration().parameters.clone();
        let parse_node = self.emit_context().parse_node(input).expect("parse node");
        let is_private = self
            .host()
            .get_effective_declaration_flags(&parse_node, ModifierFlags::Private)
            != ModifierFlags::empty();
        let new_params = self.update_accessor_param_list(input, is_private);
        Some(self.factory().update_set_accessor_declaration(
            input,
            self.ensure_modifiers(input),
            input.name().unwrap(),
            None,
            &new_params,
            None,
            None,
            None,
        ))
    }

    pub fn transform_get_accessor_declaration(&mut self, input: &Arc<Node>) -> Option<Arc<Node>> {
        if input.name().map(|n| is_private_identifier(n)).unwrap_or(false) {
            return None;
        }
        let params = input.as_get_accessor_declaration().parameters.clone();
        let parse_node = self.emit_context().parse_node(input).expect("parse node");
        let is_private = self
            .host()
            .get_effective_declaration_flags(&parse_node, ModifierFlags::Private)
            != ModifierFlags::empty();
        let new_params = self.update_accessor_param_list(input, is_private);
        let ensured_type = self.ensure_type(input, false);
        Some(self.factory().update_get_accessor_declaration(
            input,
            self.ensure_modifiers(input),
            input.name().unwrap(),
            None,
            &new_params,
            ensured_type,
            None,
            None,
        ))
    }

    pub fn update_accessor_param_list(&mut self, input: &Arc<Node>, is_private: bool) -> Arc<NodeList> {
        let mut new_params: Vec<Arc<Node>> = Vec::new();
        if !is_private {
            if let Some(this_param) = get_this_parameter(input) {
                new_params.push(self.ensure_parameter(&this_param));
            }
        }
        if is_set_accessor_declaration(input) {
            let mut value_param: Option<Arc<Node>> = None;
            let params = &input.as_set_accessor_declaration().parameters.nodes;
            if !is_private {
                if new_params.len() == 1 && params.len() >= 2 {
                    value_param = Some(self.ensure_parameter(&params[1]));
                } else if new_params.is_empty() && !params.is_empty() {
                    value_param = Some(self.ensure_parameter(&params[0]));
                }
            }
                let value_param = match value_param {
                    Some(p) => p,
                    None => {
                        let t = if !is_private {
                            Some(self.factory().new_keyword_type_node(SyntaxKind::AnyKeyword))
                        } else {
                            None
                        };
                        self.factory().new_parameter_declaration(
                            None,
                            None,
                            &self.factory().new_identifier("value"),
                            None,
                            t.as_ref(),
                            None,
                        )
                    }
                };
            new_params.push(value_param);
        }
        self.factory().new_node_list(new_params)
    }

    fn transform_method_signature_declaration(&mut self, input: &Arc<Node>) -> Option<Arc<Node>> {
        let parse_node = self.emit_context().parse_node(input).expect("parse node");
        if self
            .host()
            .get_effective_declaration_flags(&parse_node, ModifierFlags::Private)
            != ModifierFlags::empty()
        {
            self.omit_private_method_type(input)
        } else if input.name().map(|n| is_private_identifier(n)).unwrap_or(false) {
            None
        } else {
            let ms = input.as_method_signature_declaration();
            let params = ms.parameters.clone();
            let type_params = ms
                .type_parameters
                .as_ref()
                .and_then(|tp| self.ensure_type_params(input, tp));
            let new_params = self.update_param_list(input, &params);
            let ensured_type = self.ensure_type(input, false);
            Some(self.factory().update_method_signature_declaration(
                input,
                self.ensure_modifiers(input),
                input.name().unwrap().clone(),
                ms.postfix_token.clone(),
                type_params,
                new_params,
                ensured_type,
            ))
        }
    }

    pub fn transform_method_declaration(&mut self, input: &Arc<Node>) -> Option<Arc<Node>> {
        let parse_node = self.emit_context().parse_node(input).expect("parse node");
        if self
            .host()
            .get_effective_declaration_flags(&parse_node, ModifierFlags::Private)
            != ModifierFlags::empty()
        {
            self.omit_private_method_type(input)
        } else if input.name().map(|n| is_private_identifier(n)).unwrap_or(false) {
            None
        } else {
            let md = input.as_method_declaration();
            let params = md.parameters.clone();
            let type_params = md
                .type_parameters
                .as_ref()
                .and_then(|tp| self.ensure_type_params(input, tp));
            let new_params = self.update_param_list(input, &params);
            let ensured_type = self.ensure_type(input, false);
            Some(self.factory().update_method_declaration(
                input,
                self.ensure_modifiers(input),
                None,
                input.name().unwrap(),
                md.postfix_token.clone(),
                type_params,
                &new_params,
                ensured_type,
                None,
                None,
            ))
        }
    }

    fn omit_private_method_type(&mut self, input: &Arc<Node>) -> Option<Arc<Node>> {
        let declarations = input.symbol_declarations_k3();
        if !declarations.is_empty() && !Arc::ptr_eq(&declarations[0], input) {
            return None;
        }
        let result = self.factory().new_property_declaration(
            self.ensure_modifiers(input),
            input.name().unwrap(),
            None,
            None,
            None,
        );
        self.preserve_js_doc(&result, input);
        Some(result)
    }

    pub fn visit_declaration_statements(&mut self, input: Arc<Node>) -> Option<Arc<Node>> {
        if self.should_strip_internal(Some(&input)) {
            return None;
        }
        match input.kind {
            SyntaxKind::ExportDeclaration => {
                if input.parent().as_deref().map(is_source_file).unwrap_or(false) {
                    self.result_has_external_module_indicator = true;
                }
                self.result_has_scope_marker = true;
                let ed = input.as_export_declaration();
                let module_specifier = tsox_frontend::ast::mig::m3b::module_specifier(&input);
                let rewritten = self.rewrite_module_specifier(&input, module_specifier);
                Some(self.factory().update_export_declaration(
                    &input,
                    input.modifiers().cloned(),
                    ed.is_type_only,
                    ed.export_clause.clone(),
                    rewritten,
                    ed.attributes.clone(),
                ))
            }
            SyntaxKind::ExportAssignment => {
                let is_export_equals = input.as_export_assignment().is_export_equals;
                let expression = input.expression().expect("export assignment expression").clone();
                self.transform_export_assignment(&input, &input, &expression, is_export_equals)
            }
            _ => {
                let id = get_node_id(&self.emit_context().most_original(&input));
                if !self.late_statement_replacement_map.contains_key(&id) {
                    let transformed = self.transform_top_level_declaration(input.clone());
                    self.late_statement_replacement_map.insert(id, transformed);
                }
                Some(input)
            }
        }
    }

    pub fn try_get_name_of_assigned_expression(&mut self, unwrapped: &Arc<Node>) -> Option<Arc<Node>> {
        let mut name_text = String::new();
        if !is_property_access_expression(unwrapped) && unwrapped.name().is_some() {
            name_text = unwrapped.name().unwrap().text().to_string();
        } else if is_identifier(unwrapped) {
            name_text = unwrapped.text().to_string();
        }
        if !name_text.is_empty() && name_text != "default" {
            let enclosing = self.enclosing_declaration.clone();
            if self
                .resolver()
                .is_name_resolvable(enclosing.as_ref(), &name_text)
            {
                let generated = self.factory().new_unique_name_ex(
                    &name_text,
                    AutoGenerateOptions {
                        flags: GeneratedIdentifierFlags::OPTIMISTIC,
                        prefix: String::new(),
                        suffix: String::new(),
                    },
                );
                return Some(self.factory().generated_name_node(&generated));
            } else {
                return Some(self.factory().new_identifier(&name_text));
            }
        }
        None
    }

    fn get_name_of_exported_assigned_expression(
        &mut self,
        unwrapped: &Arc<Node>,
        is_export_equals: bool,
    ) -> Arc<Node> {
        let name_node = self.try_get_name_of_assigned_expression(unwrapped);
        let name_node = match name_node {
            Some(n) => n,
            None => {
                let current_source_file = self.state().current_source_file.clone();
                if is_export_equals
                    && current_source_file
                        .as_ref()
                        .and_then(|sf| crate::mig::m4m_2::registered_source_file_of_node(sf))
                        .is_some_and(|file| is_source_file_js(&file))
                {
                    self.factory().generated_name_node(&self.factory().new_unique_name_ex(
                        "_exports",
                        AutoGenerateOptions {
                            flags: GeneratedIdentifierFlags::OPTIMISTIC,
                            prefix: String::new(),
                            suffix: String::new(),
                        },
                    ))
                } else {
                    self.factory().generated_name_node(&self.factory().new_unique_name_ex(
                        "_default",
                        AutoGenerateOptions {
                            flags: GeneratedIdentifierFlags::OPTIMISTIC,
                            prefix: String::new(),
                            suffix: String::new(),
                        },
                    ))
                }
            }
        };
        self.cjs_export_assignment_name = Some(name_node.clone());
        name_node
    }

    pub fn wrap_in_cjs_export_namespace(&mut self, content: Arc<Node>) -> Arc<Node> {
        let ns_name = match &self.cjs_export_assignment_name {
            None => return content,
            Some(name) => name.clone(),
        };
        let members: Vec<Arc<Node>> = if content.kind == SyntaxKind::SyntaxList {
            content.as_syntax_list().children.clone()
        } else {
            vec![content]
        };
        let mut ns_mods: Vec<Arc<Node>> = Vec::new();
        if self.needs_declare {
            ns_mods.push(self.factory().new_modifier(SyntaxKind::DeclareKeyword));
        }
        let (members, _) = self.declare_stripping_visitor().visit_slice(members);
        self.factory().new_module_declaration(
            Some(self.factory().new_modifier_list(ns_mods)),
            SyntaxKind::NamespaceKeyword,
            &ns_name,
            None,
            Some(&self
                .factory()
                .new_module_block(self.factory().new_node_list(members))),
        )
    }

    pub fn transform_top_level_declaration(&mut self, input: Arc<Node>) -> Option<Arc<Node>> {
        if !self.state().late_marked_statements.is_empty() {
            self.state()
                .late_marked_statements
                .retain(|node| !Arc::ptr_eq(node, &input));
        }
        if self.should_strip_internal(Some(&input)) {
            return None;
        }
        if input.kind == SyntaxKind::ImportEqualsDeclaration {
            return self.transform_import_equals_declaration(&input);
        }
        if input.kind == SyntaxKind::ImportDeclaration || input.kind == SyntaxKind::JSImportDeclaration {
            let res = self.transform_import_declaration(&input);
            if let Some(res) = &res {
                if res.kind != SyntaxKind::ImportDeclaration {
                    let mut cloned = res.clone();
                    if let Some(cloned_data) = Arc::get_mut(&mut cloned) {
                        cloned_data.set_kind(SyntaxKind::ImportDeclaration);
                    }
                    return Some(cloned);
                }
            }
            return res;
        }
        if is_declaration(&input)
            && is_declaration_and_not_visible(&self.emit_context(), &self.resolver(), &input)
        {
            return None;
        }

        if is_function_like(&input) && self.resolver.is_implementation_of_overload(&input) {
            return None;
        }
        let original = self.emit_context().most_original(&input);
        let id = get_node_id(&original);
        let is_expando_host = self.expando_hosts.contains_key(&id);
        let has_deferred_expando_assignments = self.deferred_expando_assignments.contains_key(&id);
        if is_expando_host || has_deferred_expando_assignments {
            return self.create_full_expando_block(id);
        }

        let previous_enclosing_declaration = self.enclosing_declaration.clone();
        if is_enclosing_declaration(&input) {
            self.enclosing_declaration = Some(input.clone());
        }

        let can_produce_diagnostic = can_produce_diagnostics(&input);
        let old_diag = self.state().get_symbol_accessibility_diagnostic.take();
        let old_name = self.state().error_name_node.clone();
        if can_produce_diagnostic {
            self.state().get_symbol_accessibility_diagnostic =
                Some(create_get_symbol_accessibility_diagnostic_for_node(&input));
        }
        let save_needs_declare = self.needs_declare;

        let result = match input.kind {
            SyntaxKind::TypeAliasDeclaration | SyntaxKind::JSTypeAliasDeclaration => {
                self.transform_type_alias_declaration(&input)
            }
            SyntaxKind::InterfaceDeclaration => self.transform_interface_declaration(&input),
            SyntaxKind::FunctionDeclaration => self.transform_function_declaration(&input),
            SyntaxKind::ModuleDeclaration => self.transform_module_declaration(&input),
            SyntaxKind::ClassDeclaration => self.transform_class_declaration(&input),
            SyntaxKind::VariableStatement => self.transform_variable_statement(&input),
            SyntaxKind::EnumDeclaration => self.transform_enum_declaration(&input),
            _ => panic!(
                "Unhandled top-level node in declaration emit: {:?}",
                input.kind
            ),
        };

        self.enclosing_declaration = previous_enclosing_declaration;
        self.state().get_symbol_accessibility_diagnostic = old_diag;
        self.needs_declare = save_needs_declare;
        self.state().error_name_node = old_name;
        result
    }

    pub fn transform_type_alias_declaration(&mut self, input: &Arc<Node>) -> Option<Arc<Node>> {
        self.needs_declare = false;
        let ta = input.as_type_alias_declaration();
        Some(self.factory().update_type_alias_declaration(
            input,
            self.ensure_modifiers(input),
            input.name().unwrap().clone(),
            self.visitor().visit_nodes(ta.type_parameters.clone().unwrap_or_else(|| Arc::new(NodeList::new(Vec::new())))),
            self.visitor().visit(ta.type_node.clone()),
        ))
    }

    pub fn transform_interface_declaration(&mut self, input: &Arc<Node>) -> Option<Arc<Node>> {
        let id = input.as_interface_declaration();
        Some(self.factory().update_interface_declaration(
            input,
            self.ensure_modifiers(input),
            input.name().unwrap().clone(),
            self.visitor().visit_nodes(id.type_parameters.clone().unwrap_or_else(|| Arc::new(NodeList::new(Vec::new())))),
            self.visitor().visit_nodes(id.heritage_clauses.clone().unwrap_or_else(|| Arc::new(NodeList::new(Vec::new())))),
            self.visitor().visit_nodes(id.members.clone()),
        ))
    }

    pub fn transform_module_declaration(&mut self, input: &Arc<Node>) -> Option<Arc<Node>> {
        let mods = self.ensure_modifiers(input);
        let save_needs_declare = self.needs_declare;
        self.needs_declare = false;
        let inner = input.as_module_declaration().body.clone();
        let mut keyword = input.as_module_declaration().keyword;
        if keyword != SyntaxKind::GlobalKeyword
            && (input.name().is_none() || !is_string_literal(&input.name().unwrap()))
        {
            keyword = SyntaxKind::NamespaceKeyword;
        }
        let attributes = self
            .visitor()
            .visit_opt(input.as_module_declaration().attributes.clone());

        if let Some(inner) = &inner {
            if inner.kind == SyntaxKind::ModuleBlock {
                let old_needs_scope_fix = self.needs_scope_fix_marker;
                let old_has_scope_fix = self.result_has_scope_marker;
                self.result_has_scope_marker = false;
                self.needs_scope_fix_marker = false;
                let statements = match &inner.data {
                    tsox_frontend::ast::NodeData::ModuleBlock(d) => d.statements.clone(),
                    _ => Arc::new(NodeList::new(Vec::new())),
                };
                let statements = self.visitor().visit_nodes(statements);
                let mut late_statements = self.transform_and_replace_late_painted_statements(statements);
                if input.flags & NodeFlags::Ambient != NodeFlags::empty() {
                    self.needs_scope_fix_marker = false;
                }
                if !is_global_scope_augmentation(input) && !self.result_has_scope_marker
                    && !late_statements.nodes.iter().any(|s| is_scope_marker(s))
                {
                    if self.needs_scope_fix_marker {
                        let mut nodes = late_statements.nodes.clone();
                        nodes.push(create_empty_exports(&self.factory()));
                        late_statements = self.factory().new_node_list(nodes);
                    } else {
                        late_statements = self.export_stripping_visitor().visit_nodes(late_statements);
                    }
                }

                let body = self.factory().update_module_block(inner, late_statements);
                self.needs_declare = save_needs_declare;
                self.needs_scope_fix_marker = old_needs_scope_fix;
                self.result_has_scope_marker = old_has_scope_fix;

                return Some(self.factory().update_module_declaration(
                    input,
                    mods,
                    keyword,
                    input.name().cloned(),
                    attributes,
                    Some(body),
                ));
            }
            self.visitor().visit(inner.clone());
            let original = self.emit_context().most_original(inner);
            let id = get_node_id(&original);
            let body = self.late_statement_replacement_map.remove(&id).flatten();
            return Some(self.factory().update_module_declaration(
                input,
                mods,
                keyword,
                input.name().cloned(),
                attributes,
                body,
            ));
        }
        Some(self.factory().update_module_declaration(
            input,
            mods,
            keyword,
            input.name().cloned(),
            attributes,
            None,
        ))
    }

    pub fn visit_this_property_assignments(&mut self, node: Arc<Node>) -> Option<Arc<Node>> {
        let this_container = get_this_container(&node, false, false);
        let this_target = match this_container.parent() {
            None => return None,
            Some(parent) => parent,
        };
        let mut is_static = false;
        if has_static_modifier(&this_container) || is_class_static_block_declaration(&this_container) {
            is_static = true;
        }
        if !opt_arc_eq(&Some(this_target.clone()), &self.enclosing_declaration) {
            return None;
        }
        'case_block: {
            if get_assignment_declaration_kind(&node) == JsDeclarationKind::ThisProperty {
                let mut name = get_name_of_declaration(&node).unwrap_or_else(|| node.clone());
                let base = self.resolver().get_referenced_member_value_declaration(&node);
                let key = get_this_property_assignment_key(Some(&name), &node, is_static);
                let key = (
                    key.node.as_ref().map(|n| get_node_id(n)).unwrap_or(0),
                    key.is_static,
                    key.name.unwrap_or_default(),
                );
                let already_seen = base.is_none() || self.seen_properties.contains(&key);
                if already_seen {
                    break 'case_block;
                }
                self.seen_properties.insert(key);

                if !this_target.heritage_clauses().map(|h| !h.nodes.is_empty()).unwrap_or(false)
                    && !is_class_extending_null(Some(&this_target))
                {
                    self.tracker().report_inference_fallback(&this_target);
                    if self
                        .resolver()
                        .is_this_property_assignment_declaration_redundant(&node)
                    {
                        break 'case_block;
                    }
                }

                let mut mods = None;
                if is_static {
                    mods = Some(self.factory().new_modifier_list(vec![
                        self.factory().new_modifier(SyntaxKind::StaticKeyword),
                    ]));
                }
                if has_dynamic_name(&node) {
                    if !is_simple_inlineable_expression(&name) {
                        return self.this_property_visitor().visit_each_child(&node);
                    }
                    self.check_name(&node);
                    name = self.factory().new_computed_property_name(name);
                }
                if get_text_of_property_name(&name) == "constructor" {
                    return self.this_property_visitor().visit_each_child(&node);
                }
                if is_identifier(&name) && !is_identifier_text(name.text(), LanguageVariant::Standard) {
                    name = crate::mig::wt1b::r39k05_defs::new_string_literal_from_node_r39k05(
                        &self.factory(),
                        &name,
                    );
                }
                let ensured_type = self.ensure_type(&node, false);
                let prop = self.factory().new_property_declaration(
                    mods,
                    &name,
                    None,
                    ensured_type.as_ref(),
                    None,
                );
                if node.parent().as_ref().map(|p| is_expression_statement(p)).unwrap_or(false) {
                    if let Some(parent) = node.parent() {
                        self.preserve_js_doc(&prop, &parent);
                    }
                }
                self.this_property_assignments_collected.push(prop);
            }
        }
        self.this_property_visitor().visit_each_child(&node)
    }

    pub fn walk_binding_pattern(&mut self, pattern: &Arc<Node>, param: &Arc<Node>) -> Vec<Arc<Node>> {
        let mut elems: Vec<Arc<Node>> = Vec::new();
        for elem in &pattern.as_binding_pattern().elements.nodes {
            if is_omitted_expression(elem) {
                continue;
            }
            let Some(name) = elem.name() else {
                continue;
            };
            if is_binding_pattern(name) {
                elems.extend(self.walk_binding_pattern(name, param));
                continue;
            }
            let mods = self.ensure_modifiers(param);
            let ensured_type = self.ensure_type(elem, false);
            elems.push(self.factory().new_property_declaration(
                mods,
                name,
                None,
                ensured_type.as_ref(),
                None,
            ));
        }
        elems
    }

    pub fn transform_variable_statement(&mut self, input: &Arc<Node>) -> Option<Arc<Node>> {
        let declaration_list = input.as_variable_statement().declaration_list.clone();
        let declarations = &declaration_list.as_variable_declaration_list().declarations.nodes;
        let mut visible = false;
        for decl in declarations {
            visible = get_binding_name_visible(&self.resolver(), decl);
            if visible {
                break;
            }
        }
        if !visible {
            return None;
        }

        let mut input_nodes: Vec<Arc<Node>> = declarations.clone();
        let mut extra_imports: Vec<Arc<Node>> = Vec::new();
        if self.state_data.file.as_ref().map(|f| f.common_js_module_indicator.is_some()).unwrap_or(false)
        {
            let mut normal_declarations: Vec<Arc<Node>> = Vec::new();
            let mut imports: Vec<Arc<Node>> = Vec::new();
            for n in &input_nodes {
                if is_variable_declaration_initialized_to_require(n) {
                    imports.push(n.clone());
                } else {
                    normal_declarations.push(n.clone());
                }
            }
            input_nodes = normal_declarations;
            let (visited, _) = self.visitor().visit_slice(imports);
            extra_imports = visited;
        }

        let (nodes, _) = self.visitor().visit_slice(input_nodes);
        if nodes.is_empty() {
            if !extra_imports.is_empty() {
                return Some(self.factory().new_syntax_list(extra_imports));
            }
            return None;
        }
        let node_list = self.factory().new_node_list(nodes);

        let modifiers = self.ensure_modifiers(input);

        let decl_list = if is_var_using(&declaration_list) || is_var_await_using(&declaration_list) {
            let mut dl = self
                .factory()
                .new_variable_declaration_list(&node_list, NodeFlags::Const);
            self.emit_context
                .set_original(&dl, &declaration_list);
            self.emit_context
                .set_comment_range(&dl, declaration_list.loc());
            if let Some(dl_node) = Arc::get_mut(&mut dl) {
                dl_node.loc = declaration_list.loc();
            }
            dl
        } else {
            self.factory().update_variable_declaration_list(
                &declaration_list,
                &node_list,
                declaration_list.flags,
            )
        };
        let res = self
            .factory()
            .update_variable_statement(input, modifiers, decl_list);
        if !extra_imports.is_empty() {
            extra_imports.push(res);
            return Some(self.factory().new_syntax_list(extra_imports));
        }
        Some(res)
    }

    pub fn update_param_list(&mut self, node: &Arc<Node>, params: &Arc<NodeList>) -> Arc<NodeList> {
        let parse_node = self.emit_context().parse_node(node).expect("parse node");
        if self
            .host()
            .get_effective_declaration_flags(&parse_node, ModifierFlags::Private)
            != ModifierFlags::empty()
            || params.nodes.is_empty()
        {
            return self.factory().new_node_list(Vec::new());
        }
        let results: Vec<Arc<Node>> = params
            .nodes
            .iter()
            .map(|p| self.ensure_parameter(p))
            .collect();
        self.factory().new_node_list(results)
    }

    fn ensure_parameter(&mut self, p: &Arc<Node>) -> Arc<Node> {
        let old_diag = self.state().get_symbol_accessibility_diagnostic.take();
        if !self.suppress_new_diagnostic_contexts {
            self.state().get_symbol_accessibility_diagnostic =
                Some(create_get_symbol_accessibility_diagnostic_for_node(p));
        }
        let pd = p.as_parameter_declaration();
        let mut question_token = pd.question_token.clone();
        if self.resolver().is_optional_parameter(p) && question_token.is_none() {
            question_token = Some(self.factory().new_token(SyntaxKind::QuestionToken));
        }
        let visited_name = self
            .binding_name_visitor()
            .visit_node(p.name().expect("parameter name"));
        let mut updated = Node::new(
            SyntaxKind::Parameter,
            NodeData::ParameterDeclaration(ndg::ParameterDeclarationData {
                modifiers: None,
                dot_dot_dot_token: pd.dot_dot_dot_token.clone(),
                name: visited_name,
                question_token,
                type_node: self.ensure_type(p, true),
                initializer: self.ensure_no_initializer(p),
            }),
        );
        updated.loc = p.loc;
        updated.flags = p.flags;
        let result = Arc::new(updated);
        self.state().get_symbol_accessibility_diagnostic = old_diag;
        result
    }

    fn ensure_no_initializer(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        if self.should_print_with_initializer(node) {
            let initializer = node.initializer();
            let unwrapped_initializer = unwrap_parenthesized_expression(&initializer.unwrap());
            if !is_primitive_literal_value(unwrapped_initializer, true) {
                self.tracker().report_inference_fallback(node);
            }
            let parse_node = self.emit_context().parse_node(node);
            return self.resolver().create_literal_const_value(&parse_node, &self.tracker());
        }
        None
    }

    pub fn visit_binding_name(&mut self, node: Arc<Node>) -> Option<Arc<Node>> {
        match node.kind {
            SyntaxKind::Identifier | SyntaxKind::OmittedExpression => Some(node),
            SyntaxKind::ArrayBindingPattern | SyntaxKind::ObjectBindingPattern => {
                self.binding_name_visitor().visit_each_child(&node)
            }
            SyntaxKind::BindingElement => {
                let property_name = node.property_name();
                if let Some(prop_name) = &property_name {
                    if is_computed_property_name(prop_name)
                        && is_entity_name_expression(prop_name.expression().unwrap())
                    {
                        let enclosing = self.enclosing_declaration.clone();
                        self.check_entity_name_visibility(
                            prop_name.expression().unwrap(),
                            enclosing.as_ref(),
                        );
                    }
                }
                let be = node.as_binding_element();
                let visited_binding_name = self
                    .binding_name_visitor()
                    .visit_node(node.name().expect("binding name"));
                Some(self.factory().update_binding_element_r39k13(
                    &node,
                    be.dot_dot_dot_token.as_ref(),
                    property_name,
                    Some(&visited_binding_name),
                    None,
                ))
            }
            _ => Some(node),
        }
    }

    pub fn transform_import_equals_declaration(&mut self, decl: &Arc<Node>) -> Option<Arc<Node>> {
        if !self.resolver().is_declaration_visible(decl) {
            return None;
        }
        let ied = decl.as_import_equals_declaration();
        if ied.module_reference.kind == SyntaxKind::ExternalModuleReference {
            let specifier = get_external_module_import_equals_declaration_expression(decl);
            let rewritten = self.rewrite_module_specifier(decl, Some(&specifier));
            let rewritten = rewritten
                .unwrap_or_else(|| ied.module_reference.as_external_module_reference().expression.clone());
            Some(self.factory().update_import_equals_declaration(
                decl,
                decl.modifiers().cloned(),
                ied.is_type_only,
                decl.name().unwrap().clone(),
                self.factory().update_external_module_reference(&ied.module_reference, rewritten),
            ))
        } else {
            let old_diag = self.state().get_symbol_accessibility_diagnostic.take();
            self.state().get_symbol_accessibility_diagnostic =
                Some(create_get_symbol_accessibility_diagnostic_for_node(decl));
            let module_reference = ied.module_reference.clone();
            let enclosing = self.enclosing_declaration.clone();
            self.check_entity_name_visibility(&module_reference, enclosing.as_ref());
            self.state().get_symbol_accessibility_diagnostic = old_diag;
            Some(decl.clone())
        }
    }

    pub fn transform_import_declaration(&mut self, decl: &Arc<Node>) -> Option<Arc<Node>> {
        let d = decl.as_import_declaration();
        if d.import_clause.is_none() {
            let module_specifier = self
                .rewrite_module_specifier(decl, Some(&d.module_specifier))
                .unwrap_or_else(|| d.module_specifier.clone());
            return Some(self.factory().update_import_declaration(
                decl,
                decl.modifiers().cloned(),
                d.import_clause.clone(),
                module_specifier,
                d.attributes.clone(),
            ));
        }
        let import_clause = d.import_clause.clone().unwrap();
        let ic = import_clause.as_import_clause();
        let mut phase_modifier = ic.phase_modifier.unwrap_or(SyntaxKind::Unknown);
        if phase_modifier == SyntaxKind::DeferKeyword {
            phase_modifier = SyntaxKind::Unknown;
        }
        let phase_modifier = if phase_modifier == SyntaxKind::Unknown {
            None
        } else {
            Some(phase_modifier)
        };
        let mut visible_default_binding: Option<Arc<Node>> = None;
        if import_clause.name().is_some() && self.resolver().is_declaration_visible(&import_clause) {
            visible_default_binding = import_clause.name().cloned();
        }
        let named_bindings_empty = ic.named_bindings.is_none();
        if named_bindings_empty {
            if visible_default_binding.is_none() {
                return None;
            }
            let module_specifier = self
                .rewrite_module_specifier(decl, Some(&d.module_specifier))
                .unwrap_or_else(|| d.module_specifier.clone());
            return Some(self.factory().update_import_declaration(
                decl,
                decl.modifiers().cloned(),
                Some(self.factory().update_import_clause(
                    &import_clause,
                    phase_modifier,
                    visible_default_binding,
                    None,
                )),
                module_specifier,
                d.attributes.clone(),
            ));
        }
        let named_bindings = ic.named_bindings.clone().unwrap();
        if named_bindings.kind == SyntaxKind::NamespaceImport {
            let mut new_named_bindings: Option<Arc<Node>> = None;
            if self.resolver().is_declaration_visible(&named_bindings) {
                new_named_bindings = Some(named_bindings.clone());
            }
            if visible_default_binding.is_none() && new_named_bindings.is_none() {
                return None;
            }
            let module_specifier = self
                .rewrite_module_specifier(decl, Some(&d.module_specifier))
                .unwrap_or_else(|| d.module_specifier.clone());
            return Some(self.factory().update_import_declaration(
                decl,
                decl.modifiers().cloned(),
                Some(self.factory().update_import_clause(
                    &import_clause,
                    phase_modifier,
                    visible_default_binding,
                    new_named_bindings,
                )),
                module_specifier,
                d.attributes.clone(),
            ));
        }
        let named_import_elements: Vec<Arc<Node>> = match &named_bindings.data {
            tsox_frontend::ast::NodeData::NamedImports(elements) => {
                elements.elements.nodes.clone()
            }
            _ => Vec::new(),
        };
        let binding_list: Vec<Arc<Node>> = named_import_elements
            .iter()
            .filter(|b| self.resolver().is_declaration_visible(b))
            .cloned()
            .collect();
        if !binding_list.is_empty() || visible_default_binding.is_some() {
            let mut named_imports: Option<Arc<Node>> = None;
            if !binding_list.is_empty() {
                named_imports = Some(self.factory().update_named_imports(
                    &named_bindings,
                    &self.factory().new_node_list(binding_list),
                ));
            }
            let module_specifier = self
                .rewrite_module_specifier(decl, Some(&d.module_specifier))
                .unwrap_or_else(|| d.module_specifier.clone());
            return Some(self.factory().update_import_declaration(
                decl,
                decl.modifiers().cloned(),
                Some(self.factory().update_import_clause(
                    &import_clause,
                    phase_modifier,
                    visible_default_binding,
                    named_imports,
                )),
                module_specifier,
                d.attributes.clone(),
            ));
        }
        if self.resolver().is_import_required_by_augmentation(decl) {
            if self.state().isolated_declarations {
                self.state().add_diagnostic(create_diagnostic_for_node(
                    decl,
                    Some(&Diagnostics::DeclarationEmitForThisFileRequiresPreservingThisImportForAugmentationsThisIsNotSupportedWithIsolatedDeclarations),
                    &[],
                ));
            }
            let module_specifier = self
                .rewrite_module_specifier(decl, Some(&d.module_specifier))
                .unwrap_or_else(|| d.module_specifier.clone());
            return Some(self.factory().update_import_declaration(
                decl,
                decl.modifiers().cloned(),
                None,
                module_specifier,
                d.attributes.clone(),
            ));
        }
        None
    }

    pub fn transform_jsdoc_type_expression(&mut self, input: &Arc<Node>) -> Option<Arc<Node>> {
        self.visitor()
            .visit(input.as_jsdoc_type_expression().type_node.clone())
    }

    pub fn transform_jsdoc_type_literal(&mut self, input: &Arc<Node>) -> Option<Arc<Node>> {
        let (members, _) = self
            .visitor()
            .visit_slice(
                input
                    .as_jsdoc_type_literal()
                    .jsdoc_property_tags
                    .clone()
                    .unwrap_or_default(),
            );
        let replacement = self
            .factory()
            .new_type_literal_node(self.factory().new_node_list(members));
        self.emit_context().set_original(&replacement, input);
        Some(replacement)
    }

    pub fn transform_jsdoc_property_tag(&mut self, input: &Arc<Node>) -> Option<Arc<Node>> {
        let tag = input.as_jsdoc_parameter_or_property_tag();
        let replacement = self.factory().new_property_signature_declaration(
            None,
            self.visitor().visit(tag.tag_name.clone()),
            None,
            self.visitor().visit_opt(tag.type_expression.clone()),
            None,
        );
        self.emit_context().set_original(&replacement, input);
        Some(replacement)
    }

    pub fn transform_jsdoc_all_type(&mut self, input: &Arc<Node>) -> Option<Arc<Node>> {
        let replacement = self.factory().new_keyword_type_node(SyntaxKind::AnyKeyword);
        self.emit_context().set_original(&replacement, input);
        Some(replacement)
    }

    pub fn transform_jsdoc_nullable_type(&mut self, input: &Arc<Node>) -> Option<Arc<Node>> {
        let inner = self.visitor().visit(input.as_jsdoc_nullable_type().type_node.clone());
        let replacement = self.factory().new_union_type_node(self.factory().new_node_list(vec![
            inner.unwrap(),
            self.factory()
                .new_literal_type_node(self.factory().new_keyword_expression(SyntaxKind::NullKeyword)),
        ]));
        self.emit_context().set_original(&replacement, input);
        Some(replacement)
    }

    pub fn transform_jsdoc_non_nullable_type(&mut self, input: &Arc<Node>) -> Option<Arc<Node>> {
        self.visitor()
            .visit(input.as_jsdoc_non_nullable_type().type_node.clone())
    }

    pub fn transform_jsdoc_variadic_type(&mut self, input: &Arc<Node>) -> Option<Arc<Node>> {
        let inner = self.visitor()
            .visit(input.as_jsdoc_variadic_type().type_node.clone());
        let replacement = self.factory().new_array_type_node(inner);
        self.emit_context().set_original(&replacement, input);
        Some(replacement)
    }

    pub fn transform_jsdoc_optional_type(&mut self, input: &Arc<Node>) -> Option<Arc<Node>> {
        let inner = self.visitor()
            .visit(input.as_jsdoc_optional_type().type_node.clone());
        let replacement = self.factory().new_union_type_node(self.factory().new_node_list(vec![
            inner.unwrap(),
            self.factory().new_keyword_type_node(SyntaxKind::UndefinedKeyword),
        ]));
        self.emit_context().set_original(&replacement, input);
        Some(replacement)
    }

    pub fn visit_cjs_export_assignments(&mut self, expression: Option<Arc<Node>>) -> Option<Arc<Node>> {
        let expression = expression?;
        let (_, cleanup_diagnostic_context) = self.setup_diagnostic_context(&expression);
        if get_assignment_declaration_kind(&expression) == JsDeclarationKind::ModuleExports {
            let has_indicator = self
                .state_data
                .file
                .as_ref()
                .map(|f| f.common_js_module_indicator.is_some())
                .unwrap_or(false);
            if has_indicator {
                let right = expression.as_binary_expression().right.clone();
                let result = self.transform_export_assignment(
                    &expression.parent().unwrap(),
                    &expression,
                    &right,
                    true,
                );
                if let Some(result) = result {
                    self.cjs_export_assignment = Some(result);
                    self.result_has_scope_marker = true;
                    self.result_has_external_module_indicator = true;
                }
            }
        }
        cleanup_diagnostic_context(self);
        self.cjs_export_assignment_visitor().visit_each_child(&expression)
    }

    pub fn visit_nested_expression(&mut self, expression: Option<Arc<Node>>) -> Option<Arc<Node>> {
        let expression = expression?;
        let (_, cleanup_diagnostic_context) = self.setup_diagnostic_context(&expression);
        match get_assignment_declaration_kind(&expression) {
            JsDeclarationKind::Property => {
                self.transform_expando_assignment(&expression);
            }
            JsDeclarationKind::ExportsProperty => {
                let has_indicator = self
                    .state_data
                    .file
                    .as_ref()
                    .map(|f| f.common_js_module_indicator.is_some())
                    .unwrap_or(false);
                if has_indicator {
                    let left = expression.as_binary_expression().left.clone();
                    let name = get_element_or_property_access_name(&left);
                    let name = name.map(|n| self.get_name_expression_preferring_identifier(&n));
                    let result = self.transform_common_js_export(&expression, name);
                    if let Some(result) = result {
                        self.cjs_export_members.push(result);
                    }
                }
            }
            JsDeclarationKind::ObjectDefinePropertyExports => {
                let has_indicator = self
                    .state_data
                    .file
                    .as_ref()
                    .map(|f| f.common_js_module_indicator.is_some())
                    .unwrap_or(false);
                if has_indicator {
                    let args = expression.arguments();
                    let name = self.get_name_expression_preferring_identifier(args.as_ref().map(|l|&l.nodes[1]).unwrap());
                    let result = self.transform_common_js_export(&expression, Some(name));
                    if let Some(result) = result {
                        self.cjs_export_members.push(result);
                    }
                }
            }
            _ => {}
        }
        cleanup_diagnostic_context(self);
        self.expression_visitor().visit_each_child(&expression)
    }

    pub fn transform_expando_assignment(&mut self, node: &Arc<Node>) {
        let left = node.as_binary_expression().left.clone();
        // Go 2719-2723: node.Symbol 含 SymbolFlagsAssignment 才继续。Node→Symbol 接线
        // (m3c::symbol 需 NodeSymbolMap)属 checker 裁决项,接线前无法判定;
        // 依赖 symbol 的后半段(Go 2744 host symbol 与 Go 2787 namespace symbol/locals
        // 接线)与 tx.transformExpandoHost(Go 2794)同留 r43 交接。
        let ns = get_leftmost_access_expression(&left);
        if ns.kind != SyntaxKind::Identifier {
            return;
        }
        let declaration = match self.resolver().get_referenced_value_declaration_unsafe(&ns) {
            Some(d) => d,
            None => return,
        };
        if self.should_strip_internal(Some(&declaration)) {
            return;
        }
        if is_variable_declaration(&declaration) && declaration.type_().is_some() {
            return;
        }
        if is_function_declaration(&declaration)
            && declaration
                .as_function_declaration_r42k01()
                .full_signature
                .is_some()
        {
            return;
        }
        if is_variable_declaration(&declaration) {
            let initializer_is_function_like = declaration
                .initializer()
                .map(|i| is_function_like(&i))
                .unwrap_or(false);
            if !initializer_is_function_like {
                return;
            }
        }
        let name = self.factory().new_identifier(ns.text());
        let property = self.try_get_property_name(&left);
        if property.is_empty() || !is_identifier_text(&property, LanguageVariant::Standard) {
            return;
        }
        let host_root = if is_variable_declaration(&declaration) {
            declaration.parent().and_then(|p| p.parent())
        } else {
            Some(declaration.clone())
        };
        let Some(host_root) = host_root else {
            return;
        };
        let host_id = get_node_id(&self.emit_context().most_original(&host_root));
        if is_declaration(&declaration)
            && is_declaration_and_not_visible(&self.emit_context(), &self.resolver(), &declaration)
        {
            self.deferred_expando_assignments
                .entry(host_id)
                .or_default()
                .push(node.clone());
            return;
        }
        if is_function_declaration(&declaration) && !should_emit_function_properties(&declaration) {
            return;
        }
        let export_name = self.factory().new_identifier(&property);
        let local_name = self.try_get_name_of_assigned_expression(node);
        let enclosing = self.enclosing_declaration.clone();
        let local_name = match local_name {
            Some(local_name) => {
                if is_identifier_text(local_name.text(), LanguageVariant::Standard) {
                    Some(local_name)
                } else {
                    None
                }
            }
            None => {
                if !self.resolver().is_name_resolvable(enclosing.as_ref(), &property)
                    && is_identifier_text(&property, LanguageVariant::Standard)
                {
                    Some(export_name.clone())
                } else {
                    None
                }
            }
        };
        let local_name = match local_name {
            Some(local_name) => local_name,
            None => {
                let generated =
                    self.factory()
                        .new_unique_name_ex(&property, AutoGenerateOptions {
                            flags: GeneratedIdentifierFlags::OPTIMISTIC,
                            prefix: String::new(),
                            suffix: String::new(),
                        });
                self.factory().generated_name_node(&generated)
            }
        };
        let (_, cleanup_diagnostic_context) = self.setup_diagnostic_context(node);
        if is_identifier(node.as_binary_expression().right.as_ref()) {
            let result =
                self.transform_binary_expression_to_export_declaration(node, &export_name);
            if let Some(result) = result {
                self.expando_members.entry(host_id).or_default().push(result);
            }
            cleanup_diagnostic_context(self);
            return;
        }
        let ensured_type = self.ensure_type(node, false);
        let var_decl = self.factory().new_variable_declaration(
            &local_name,
            None,
            ensured_type.as_ref(),
            None,
        );
        let decl_list = self.factory().new_variable_declaration_list(
            &self.factory().new_node_list(vec![var_decl]),
            NodeFlags::empty(),
        );
        let statement = self.factory().new_variable_statement_r42k01(None, decl_list);
        let mut statements = vec![statement];
        if local_name.text() != export_name.text() {
            let export_specifier =
                self.factory()
                    .new_export_specifier(false, Some(&local_name), &export_name);
            let named_exports = self
                .factory()
                .new_named_exports(&self.factory().new_node_list(vec![export_specifier]));
            let export_decl = self.factory().new_export_declaration(
                None,
                false,
                &named_exports,
                None,
                None,
            );
            statements.push(export_decl);
        }
        self.expando_members
            .entry(host_id)
            .or_default()
            .extend(statements);
        cleanup_diagnostic_context(self);
    }

    pub fn transform_common_js_export(
        &mut self,
        input: &Arc<Node>,
        name: Option<Arc<Node>>,
    ) -> Option<Arc<Node>> {
        let res = self.transform_common_js_export_worker(input, name)?;
        Some(self.wrap_in_cjs_export_namespace(res))
    }

    fn transform_common_js_export_worker(
        &mut self,
        input: &Arc<Node>,
        name: Option<Arc<Node>>,
    ) -> Option<Arc<Node>> {
        let mut name_text = String::new();
        if let Some(name) = &name {
            if is_identifier(name) || is_string_literal(name) {
                name_text = name.text().to_string();
            }
        }
        if !name_text.is_empty() && self.witnessed_cjs_exports.contains(&name_text) {
            return None;
        }
        self.witnessed_cjs_exports.insert(name_text);
        self.result_has_external_module_indicator = true;
        self.result_has_scope_marker = true;
        // Go 1354-1356: isCommonJSAliasExport(input) 命中且父链为顶层表达式语句时,
        // 走 transformBinaryExpressionToExportDeclaration 别名短路路径。该判定需 node
        // symbol(x6a::is_common_js_alias_export 要求 NodeSymbolMap,未接线即 Go
        // symbol==nil → false),Node→Symbol 接线属 checker 裁决项 [r43 交接],
        // 接线后在此处补:parent 为 ExpressionStatement 且 grandparent 为 SourceFile
        // 时 return transform_binary_expression_to_export_declaration(input, name)。
        // Go 1361-1449: RHS 为 class expression 的提升路径依赖
        // transform_class_expression_to_declaration,缺,留交接(同 transform_export_assignment 注释)。
        let Some(name) = name else {
            return None;
        };
        if is_identifier(&name) && name.text() == "default" {
            let new_id = self.factory().generated_name_node(&self.factory().new_unique_name_ex(
                "_default",
                AutoGenerateOptions {
                    flags: GeneratedIdentifierFlags::OPTIMISTIC,
                    prefix: String::new(),
                    suffix: String::new(),
                },
            ));
            let diag_error_node = input.clone();
            self.state().get_symbol_accessibility_diagnostic = Some(Box::new(
                move |_result: &tsox_checker::checker::types::SymbolAccessibilityResult| {
                    Some(crate::mig::m3n::SymbolAccessibilityDiagnostic {
                        diagnostic_message: &tsox_core::diagnostics::messages_generated::DEFAULT_EXPORT_OF_THE_MODULE_HAS_OR_IS_USING_PRIVATE_NAME_0,
                        error_node: Some(diag_error_node.clone()),
                        type_name: None,
                    })
                },
            ));
            let ensured_type = self.ensure_type(input, false);
            let var_decl =
                self.factory()
                    .new_variable_declaration(&new_id, None, ensured_type.as_ref(), None);
            let mod_list = if self.needs_declare {
                self.factory().new_modifier_list(vec![self
                    .factory()
                    .new_modifier(SyntaxKind::DeclareKeyword)])
            } else {
                self.factory().new_modifier_list(Vec::new())
            };
            let decl_list = self.factory().new_variable_declaration_list(
                &self.factory().new_node_list(vec![var_decl]),
                NodeFlags::Const,
            );
            let statement =
                self.factory()
                    .new_variable_statement_r42k01(Some(mod_list), decl_list);
            let assignment = self
                .factory()
                .new_export_assignment(input.modifiers().cloned(), false, None, &new_id);
            self.preserve_js_doc(&statement, input);
            self.emit_context()
                .add_emit_flags(&assignment, EmitFlags::NO_COMMENTS);
            return Some(self.factory().new_syntax_list(vec![statement, assignment]));
        }
        if is_identifier(&name) {
            let referenced = self.resolver().get_referenced_value_declaration_unsafe(&name);
            let referenced_points_here = match referenced {
                Some(d) => Arc::ptr_eq(&d, &name),
                None => true,
            };
            if referenced_points_here {
                let ensured_type = self.ensure_type(input, false);
                let var_decl =
                    self.factory()
                        .new_variable_declaration(&name, None, ensured_type.as_ref(), None);
                let mod_list = if self.needs_declare {
                    self.factory().new_modifier_list(vec![
                        self.factory().new_modifier(SyntaxKind::ExportKeyword),
                        self.factory().new_modifier(SyntaxKind::DeclareKeyword),
                    ])
                } else {
                    self.factory().new_modifier_list(vec![self
                        .factory()
                        .new_modifier(SyntaxKind::ExportKeyword)])
                };
                let decl_list = self.factory().new_variable_declaration_list(
                    &self.factory().new_node_list(vec![var_decl]),
                    NodeFlags::empty(),
                );
                return Some(self.factory().new_variable_statement_r42k01(
                    Some(mod_list),
                    decl_list,
                ));
            }
        }
        let new_id = self.factory().generated_name_node(&self.factory().new_unique_name_ex(
            "_exported",
            AutoGenerateOptions {
                flags: GeneratedIdentifierFlags::OPTIMISTIC,
                prefix: String::new(),
                suffix: String::new(),
            },
        ));
        let diag_error_node = input.clone();
        self.state().get_symbol_accessibility_diagnostic = Some(Box::new(
            move |_result: &tsox_checker::checker::types::SymbolAccessibilityResult| {
                Some(crate::mig::m3n::SymbolAccessibilityDiagnostic {
                    diagnostic_message: &tsox_core::diagnostics::messages_generated::DEFAULT_EXPORT_OF_THE_MODULE_HAS_OR_IS_USING_PRIVATE_NAME_0,
                    error_node: Some(diag_error_node.clone()),
                    type_name: None,
                })
            },
        ));
        let ensured_type = self.ensure_type(input, false);
        let var_decl = self.factory().new_variable_declaration(
            &new_id,
            None,
            ensured_type.as_ref(),
            None,
        );
        let mod_list = if self.needs_declare {
            self.factory()
                .new_modifier_list(vec![self.factory().new_modifier(SyntaxKind::DeclareKeyword)])
        } else {
            self.factory().new_modifier_list(Vec::new())
        };
        let decl_list = self.factory().new_variable_declaration_list(
            &self.factory().new_node_list(vec![var_decl]),
            NodeFlags::Const,
        );
        let statement = self
            .factory()
            .new_variable_statement_r42k01(Some(mod_list), decl_list);
        let export_specifier = self.factory().new_export_specifier(false, Some(&new_id), &name);
        let named_exports = self
            .factory()
            .new_named_exports(&self.factory().new_node_list(vec![export_specifier]));
        let assignment = self
            .factory()
            .new_export_declaration(None, false, &named_exports, None, None);
        self.preserve_js_doc(&statement, input);
        self.emit_context()
            .add_emit_flags(&assignment, EmitFlags::NO_COMMENTS);
        Some(self.factory().new_syntax_list(vec![statement, assignment]))
    }

    pub fn transform_binary_expression_to_export_declaration(
        &mut self,
        input: &Arc<Node>,
        name: &Arc<Node>,
    ) -> Option<Arc<Node>> {
        let property_name = input.as_binary_expression().right.clone();
        // Go 1317: tx.tracker.handleSymbolAccessibilityError(
        //   resolver.IsEntityNameVisible(propertyName, enclosingDeclaration));
        // 本 crate EmitTracker trait 无 handle_symbol_accessibility_error 回调,留 r43 交接。
        let property_name = if is_identifier(name) && property_name.text() == name.text() {
            None
        } else {
            Some(property_name)
        };
        let export_specifier =
            self.factory()
                .new_export_specifier(false, property_name.as_ref(), name);
        let named_exports = self
            .factory()
            .new_named_exports(&self.factory().new_node_list(vec![export_specifier]));
        Some(self.factory().new_export_declaration(
            None,
            false,
            &named_exports,
            None,
            None,
        ))
    }

    pub fn try_get_property_name(&mut self, node: &Arc<Node>) -> String {
        if is_element_access_expression(node) {
            return self.resolver().get_element_access_expression_name(node);
        }
        if is_property_access_expression(node) {
            return node.name().map(|n| n.text().to_string()).unwrap_or_default();
        }
        String::new()
    }

    pub fn get_referenced_files(&self, _output_file_path: &str) -> Vec<FileReference> {
        self.state_data
            .raw_referenced_files
            .iter()
            .filter(|pair| pair.r#ref.preserve)
            .map(|pair| FileReference {
                text_range: pair.r#ref.text_range,
                file_name: pair.r#ref.file_name.clone(),
                resolution_mode: pair.r#ref.resolution_mode,
                preserve: pair.r#ref.preserve,
            })
            .collect()
    }

    pub fn create_full_expando_block(&mut self, id: u64) -> Option<Arc<Node>> {
        let n = self.expando_hosts.get(&id).cloned();
        if let Some(add_ons) = self.expando_members.remove(&id) {
            let mut modifiers: Option<Arc<ModifierList>> = None;
            let mut name: Option<Arc<Node>> = None;
            let mut host: Vec<Arc<Node>> = Vec::new();
            if let Some(n) = &n {
                if n.kind == SyntaxKind::SyntaxList {
                    for c in node_or_syntax_list_children(n) {
                        if let Some(c_name) = c.name() {
                            name = Some(tsox_frontend::ast::deep_clone_node(c_name));
                            if let Some(c_modifiers) = c.modifiers() {
                                modifiers = Some(Arc::new(deep_clone_modifier_list_r42k01(
                                    c_modifiers,
                                )));
                            }
                            break;
                        }
                    }
                    host = node_or_syntax_list_children(n);
                } else {
                    name = n.name().map(tsox_frontend::ast::deep_clone_node);
                    if let Some(n_modifiers) = n.modifiers() {
                        modifiers =
                            Some(Arc::new(deep_clone_modifier_list_r42k01(n_modifiers)));
                    }
                    host = vec![n.clone()];
                }
            }
            if let Some(name) = name {
                let module_block = self
                    .factory()
                    .new_module_block(self.factory().new_node_list(add_ons));
                let module_decl = self.factory().new_module_declaration(
                    modifiers,
                    SyntaxKind::NamespaceKeyword,
                    &name,
                    None,
                    Some(&module_block),
                );
                let mut members = host;
                members.push(module_decl);
                return Some(self.factory().new_syntax_list(members));
            }
        }
        n
    }

    pub fn transform_function_declaration(&mut self, input: &Arc<Node>) -> Option<Arc<Node>> {
        let fd = input.as_function_declaration_r42k01();
        let type_parameters = fd
            .type_parameters
            .clone()
            .and_then(|tp| self.ensure_type_params(input, &tp));
        let params = fd.parameters.clone();
        let new_params = self.update_param_list(input, &params);
        let ensured_type = self.ensure_type(input, false);
        Some(self.factory().update_function_declaration(
            input,
            self.ensure_modifiers(input),
            None,
            fd.name.as_ref(),
            type_parameters,
            &new_params,
            ensured_type,
            None,
            None,
        ))
    }

    pub fn transform_class_declaration(&mut self, input: &Arc<Node>) -> Option<Arc<Node>> {
        let previous_enclosing_declaration = self.enclosing_declaration.clone();
        self.enclosing_declaration = Some(input.clone());
        let old_name = self.state().error_name_node.clone();
        self.state().error_name_node = input.name().cloned();

        let cd = input.as_class_declaration_r42k01();
        let type_parameters = cd
            .type_parameters
            .clone()
            .and_then(|tp| self.ensure_type_params(input, &tp));
        let heritage_clauses = cd
            .heritage_clauses
            .clone()
            .map(|h| self.visitor().visit_nodes(h));
        let members = self.visitor().visit_nodes(cd.members.clone());
        let result = self.factory().update_class_declaration(
            input,
            self.ensure_modifiers(input),
            cd.name.as_ref(),
            type_parameters,
            heritage_clauses.as_deref(),
            &members,
        );

        self.state().error_name_node = old_name;
        self.enclosing_declaration = previous_enclosing_declaration;
        Some(result)
    }

    pub fn transform_enum_declaration(&mut self, input: &Arc<Node>) -> Option<Arc<Node>> {
        let ed = input.as_enum_declaration_r42k01();
        let kept: Vec<Arc<Node>> = ed
            .members
            .nodes
            .iter()
            .filter(|m| !self.should_strip_internal(Some(m)))
            .cloned()
            .collect();
        let members = self.factory().new_node_list(kept);
        Some(self.factory().update_enum_declaration_r42k01(
            input,
            self.ensure_modifiers(input),
            input.name().unwrap(),
            members,
        ))
    }

    pub fn transform_call_signature_declaration(&mut self, input: &Arc<Node>) -> Option<Arc<Node>> {
        let csd = input.as_call_signature_declaration_r42k01();
        let type_parameters = csd
            .type_parameters
            .clone()
            .and_then(|tp| self.ensure_type_params(input, &tp));
        let params = csd.parameters.clone();
        let new_params = self.update_param_list(input, &params);
        let ensured_type = self.ensure_type(input, false);
        Some(self.factory().update_call_signature_declaration(
            input,
            type_parameters,
            new_params,
            ensured_type,
        ))
    }

    pub fn transform_construct_signature_declaration(
        &mut self,
        input: &Arc<Node>,
    ) -> Option<Arc<Node>> {
        let csd = input.as_construct_signature_declaration_r42k01();
        let type_parameters = csd
            .type_parameters
            .clone()
            .and_then(|tp| self.ensure_type_params(input, &tp));
        let params = csd.parameters.clone();
        let new_params = self.update_param_list(input, &params);
        let ensured_type = self.ensure_type(input, false);
        Some(self.factory().update_construct_signature_declaration(
            input,
            type_parameters,
            new_params,
            ensured_type,
        ))
    }

    pub fn transform_constructor_declaration(&mut self, input: &Arc<Node>) -> Option<Arc<Node>> {
        let ctor = input.as_constructor_declaration_r42k01();
        let params = ctor.parameters.clone();
        let new_params = self.update_param_list(input, &params);
        Some(self.factory().update_constructor_declaration(
            input,
            self.ensure_modifiers(input),
            None,
            &new_params,
            None,
            None,
            None,
        ))
    }

    pub fn transform_conditional_type_node(&mut self, input: &Arc<Node>) -> Option<Arc<Node>> {
        let ctd = input.as_conditional_type_node_r42k01();
        let check_type = self
            .visitor()
            .visit(ctd.check_type.clone())
            .unwrap_or_else(|| ctd.check_type.clone());
        let extends_type = self
            .visitor()
            .visit(ctd.extends_type.clone())
            .unwrap_or_else(|| ctd.extends_type.clone());
        let old_enclosing_declaration = self.enclosing_declaration.clone();
        self.enclosing_declaration = Some(ctd.true_type.clone());
        let true_type = self
            .visitor()
            .visit(ctd.true_type.clone())
            .unwrap_or_else(|| ctd.true_type.clone());
        self.enclosing_declaration = old_enclosing_declaration;
        let false_type = self
            .visitor()
            .visit(ctd.false_type.clone())
            .unwrap_or_else(|| ctd.false_type.clone());
        Some(self.factory().update_conditional_type_node(
            input,
            check_type,
            extends_type,
            true_type,
            false_type,
        ))
    }

    pub fn transform_function_type_node(&mut self, input: &Arc<Node>) -> Option<Arc<Node>> {
        let ftd = input.as_function_type_node_r42k01();
        let type_parameters = ftd
            .type_parameters
            .clone()
            .map(|tp| self.visitor().visit_nodes(tp));
        let params = ftd.parameters.clone();
        let new_params = self.update_param_list(input, &params);
        let type_node = ftd
            .type_node
            .clone()
            .and_then(|t| self.visitor().visit(t));
        Some(self.factory().update_function_type_node(
            input,
            type_parameters,
            new_params,
            type_node,
        ))
    }

    pub fn transform_constructor_type_node(&mut self, input: &Arc<Node>) -> Option<Arc<Node>> {
        let ctd = input.as_constructor_type_node_r42k01();
        let type_parameters = ctd
            .type_parameters
            .clone()
            .map(|tp| self.visitor().visit_nodes(tp));
        let params = ctd.parameters.clone();
        let new_params = self.update_param_list(input, &params);
        let type_node = ctd
            .type_node
            .clone()
            .and_then(|t| self.visitor().visit(t));
        Some(self.factory().update_constructor_type_node(
            input,
            self.ensure_modifiers(input),
            type_parameters,
            new_params,
            type_node,
        ))
    }

    pub fn transform_export_assignment(
        &mut self,
        input: &Arc<Node>,
        assignment: &Arc<Node>,
        expression: &Arc<Node>,
        is_export_equals: bool,
    ) -> Option<Arc<Node>> {
        let parent_is_source_file = input
            .parent()
            .as_deref()
            .map(is_source_file)
            .unwrap_or(false);
        let parent_is_module_block = input
            .parent()
            .as_deref()
            .map(|p| p.kind == SyntaxKind::ModuleBlock)
            .unwrap_or(false);
        if parent_is_source_file {
            self.result_has_external_module_indicator = true;
        }
        self.result_has_scope_marker = true;
        if is_identifier(expression) && (parent_is_source_file || parent_is_module_block) {
            let export_assignment =
                self.factory()
                    .new_export_assignment(None, is_export_equals, None, expression);
            self.preserve_js_doc(&export_assignment, input);
            return Some(export_assignment);
        }

        // Go 1228-1259: class/function 表达式提升路径依赖
        // transform_class_expression_to_declaration / transform_function_like_to_declaration,均缺,留交接
        let diag_error_node = input.clone();
        self.state().get_symbol_accessibility_diagnostic =
            Some(Box::new(move |_result: &tsox_checker::checker::types::SymbolAccessibilityResult| {
                Some(crate::mig::m3n::SymbolAccessibilityDiagnostic {
                    diagnostic_message: &tsox_core::diagnostics::messages_generated::DEFAULT_EXPORT_OF_THE_MODULE_HAS_OR_IS_USING_PRIVATE_NAME_0,
                    error_node: Some(diag_error_node.clone()),
                    type_name: None,
                })
            }));
        let new_id = self.get_name_of_exported_assigned_expression(expression, is_export_equals);
        self.cjs_export_assignment_name = Some(new_id.clone());
        let initializer = if is_primitive_literal_value(
            unwrap_parenthesized_expression(expression),
            true,
        ) {
            let parse_node = self.emit_context().parse_node(assignment);
            self.resolver()
                .create_literal_const_value(&parse_node, &self.tracker())
        } else {
            None
        };
        let type_node = if initializer.is_none() {
            self.ensure_type(assignment, false)
        } else {
            None
        };
        let var_decl = self.factory().new_variable_declaration(
            &new_id,
            None,
            type_node.as_ref(),
            initializer.as_ref(),
        );
        let mod_list = if self.needs_declare {
            self.factory()
                .new_modifier_list(vec![self.factory().new_modifier(SyntaxKind::DeclareKeyword)])
        } else {
            self.factory().new_modifier_list(Vec::new())
        };
        let decl_list = self.factory().new_variable_declaration_list(
            &self.factory().new_node_list(vec![var_decl]),
            NodeFlags::Const,
        );
        let statement =
            self.factory()
                .new_variable_statement_r42k01(Some(mod_list), decl_list);
        let export_assignment =
            self.factory()
                .new_export_assignment(None, is_export_equals, None, &new_id);
        self.preserve_js_doc(&statement, input);
        Some(self.factory().new_syntax_list(vec![statement, export_assignment]))
    }
}

pub trait R42K01DataExt {
    fn as_function_declaration_r42k01(&self) -> &ndg::FunctionDeclarationData;
    fn as_class_declaration_r42k01(&self) -> &ndg::ClassDeclarationData;
    fn as_enum_declaration_r42k01(&self) -> &ndg::EnumDeclarationData;
    fn as_call_signature_declaration_r42k01(&self) -> &ndg::CallSignatureDeclarationData;
    fn as_construct_signature_declaration_r42k01(&self) -> &ndg::ConstructSignatureDeclarationData;
    fn as_constructor_declaration_r42k01(&self) -> &ndg::ConstructorDeclarationData;
    fn as_conditional_type_node_r42k01(&self) -> &ndg::ConditionalTypeNodeData;
    fn as_function_type_node_r42k01(&self) -> &ndg::FunctionTypeNodeData;
    fn as_constructor_type_node_r42k01(&self) -> &ndg::ConstructorTypeNodeData;
}

macro_rules! r42k01_as_data {
    ($name:ident, $variant:ident, $ty:ty) => {
        fn $name(&self) -> &$ty {
            match &self.data {
                NodeData::$variant(d) => d,
                _ => panic!(concat!("As", stringify!($variant), " on wrong node kind")),
            }
        }
    };
}

impl R42K01DataExt for Node {
    r42k01_as_data!(as_function_declaration_r42k01, FunctionDeclaration, ndg::FunctionDeclarationData);
    r42k01_as_data!(as_class_declaration_r42k01, ClassDeclaration, ndg::ClassDeclarationData);
    r42k01_as_data!(as_enum_declaration_r42k01, EnumDeclaration, ndg::EnumDeclarationData);
    r42k01_as_data!(as_call_signature_declaration_r42k01, CallSignatureDeclaration, ndg::CallSignatureDeclarationData);
    r42k01_as_data!(as_construct_signature_declaration_r42k01, ConstructSignatureDeclaration, ndg::ConstructSignatureDeclarationData);
    r42k01_as_data!(as_constructor_declaration_r42k01, ConstructorDeclaration, ndg::ConstructorDeclarationData);
    r42k01_as_data!(as_conditional_type_node_r42k01, ConditionalTypeNode, ndg::ConditionalTypeNodeData);
    r42k01_as_data!(as_function_type_node_r42k01, FunctionTypeNode, ndg::FunctionTypeNodeData);
    r42k01_as_data!(as_constructor_type_node_r42k01, ConstructorTypeNode, ndg::ConstructorTypeNodeData);
}

pub trait R42K01EmitResolverExt {
    fn precalculate_declaration_emit_visibility(&self, file: &tsox_frontend::ast::SourceFile);
}

impl R42K01EmitResolverExt for EmitResolver {
    fn precalculate_declaration_emit_visibility(&self, _file: &tsox_frontend::ast::SourceFile) {
        // m2d EmitResolver 的可见性判定均为无状态查询,无预计算缓存可落
    }
}

impl<'a> NodeFactory<'a> {
    pub fn update_call_signature_declaration(
        &self,
        node: &Arc<Node>,
        type_parameters: Option<Arc<NodeList>>,
        parameters: Arc<NodeList>,
        type_node: Option<Arc<Node>>,
    ) -> Arc<Node> {
        let mut updated = Node::new(
            SyntaxKind::CallSignature,
            NodeData::CallSignatureDeclaration(ndg::CallSignatureDeclarationData {
                type_parameters,
                parameters,
                type_node,
            }),
        );
        updated.loc = node.loc;
        updated.flags = node.flags;
        Arc::new(updated)
    }

    pub fn update_construct_signature_declaration(
        &self,
        node: &Arc<Node>,
        type_parameters: Option<Arc<NodeList>>,
        parameters: Arc<NodeList>,
        type_node: Option<Arc<Node>>,
    ) -> Arc<Node> {
        let mut updated = Node::new(
            SyntaxKind::ConstructSignature,
            NodeData::ConstructSignatureDeclaration(ndg::ConstructSignatureDeclarationData {
                type_parameters,
                parameters,
                type_node,
            }),
        );
        updated.loc = node.loc;
        updated.flags = node.flags;
        Arc::new(updated)
    }

    pub fn update_conditional_type_node(
        &self,
        node: &Arc<Node>,
        check_type: Arc<Node>,
        extends_type: Arc<Node>,
        true_type: Arc<Node>,
        false_type: Arc<Node>,
    ) -> Arc<Node> {
        let mut updated = Node::new(
            SyntaxKind::ConditionalType,
            NodeData::ConditionalTypeNode(ndg::ConditionalTypeNodeData {
                check_type,
                extends_type,
                true_type,
                false_type,
            }),
        );
        updated.loc = node.loc;
        updated.flags = node.flags;
        Arc::new(updated)
    }

    pub fn update_function_type_node(
        &self,
        node: &Arc<Node>,
        type_parameters: Option<Arc<NodeList>>,
        parameters: Arc<NodeList>,
        type_node: Option<Arc<Node>>,
    ) -> Arc<Node> {
        let mut updated = Node::new(
            SyntaxKind::FunctionType,
            NodeData::FunctionTypeNode(ndg::FunctionTypeNodeData {
                type_parameters,
                parameters,
                type_node,
            }),
        );
        updated.loc = node.loc;
        updated.flags = node.flags;
        Arc::new(updated)
    }

    pub fn update_constructor_type_node(
        &self,
        node: &Arc<Node>,
        modifiers: Option<Arc<ModifierList>>,
        type_parameters: Option<Arc<NodeList>>,
        parameters: Arc<NodeList>,
        type_node: Option<Arc<Node>>,
    ) -> Arc<Node> {
        let mut updated = Node::new(
            SyntaxKind::ConstructorType,
            NodeData::ConstructorTypeNode(ndg::ConstructorTypeNodeData {
                modifiers,
                type_parameters,
                parameters,
                type_node,
            }),
        );
        updated.loc = node.loc;
        updated.flags = node.flags;
        Arc::new(updated)
    }

    pub fn update_enum_declaration_r42k01(
        &self,
        node: &Arc<Node>,
        modifiers: Option<Arc<ModifierList>>,
        name: &Arc<Node>,
        members: Arc<NodeList>,
    ) -> Arc<Node> {
        let mut updated = Node::new(
            SyntaxKind::EnumDeclaration,
            NodeData::EnumDeclaration(ndg::EnumDeclarationData {
                modifiers,
                name: name.clone(),
                members,
            }),
        );
        updated.loc = node.loc;
        updated.flags = node.flags;
        Arc::new(updated)
    }

    pub fn new_variable_statement_r42k01(
        &self,
        modifiers: Option<Arc<ModifierList>>,
        declaration_list: Arc<Node>,
    ) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::VariableStatement,
            NodeData::VariableStatement(ndg::VariableStatementData {
                modifiers,
                declaration_list,
            }),
        ))
    }
}

fn deep_clone_modifier_list_r42k01(modifiers: &Arc<ModifierList>) -> ModifierList {
    ModifierList::new(
        modifiers
            .list
            .nodes
            .iter()
            .map(|n| tsox_frontend::ast::deep_clone_node(n))
            .collect(),
        modifiers.modifier_flags,
    )
}

pub struct ReferencedFilePair {
    pub file: Arc<Node>,
    pub r#ref: FileReference,
}

pub fn node_or_syntax_list_children(node: &Arc<Node>) -> Vec<Arc<Node>> {
    if is_syntax_list(node) {
        return node.as_syntax_list().children.clone();
    }
    vec![node.clone()]
}

pub fn flatten_syntax_lists(nodes: &[Arc<Node>]) -> Vec<Arc<Node>> {
    nodes
        .iter()
        .flat_map(node_or_syntax_list_children)
        .collect()
}

pub fn get_this_parameter(signature: &Arc<Node>) -> Option<Arc<Node>> {
    let parameters = tsox_frontend::ast::mig::m3b::parameters(signature);
    if !parameters.is_empty() {
        let this_parameter = &parameters[0];
        if is_this_parameter(this_parameter) {
            return Some(this_parameter.clone());
        }
    }
    None
}

pub fn create_empty_exports(factory: &NodeFactory) -> Arc<Node> {
    factory.new_export_declaration(
        None,
        false,
        &factory.new_named_exports(&factory.new_node_list(Vec::new())),
        None,
        None,
    )
}

pub fn has_any_binding_initializers(binding_pattern: &Arc<Node>) -> bool {
    for elem in &binding_pattern.as_binding_pattern().elements.nodes {
        if !is_binding_element(elem) {
            continue;
        }
        let e = elem.as_binding_element();
        if e.initializer.is_some() {
            return true;
        }
        if let Some(name) = elem.name() {
            if name.kind != SyntaxKind::Unknown
                && is_binding_pattern(name)
                && has_any_binding_initializers(name)
            {
                return true;
            }
        }
    }
    false
}

pub fn is_class_extending_null(node: Option<&Arc<Node>>) -> bool {
    let node = match node {
        None => return false,
        Some(n) => n,
    };
    let extends_clause = get_heritage_clause(node, SyntaxKind::ExtendsKeyword);
    let extends_clause = match extends_clause {
        None => return false,
        Some(c) => c,
    };
    let types = &extends_clause.as_heritage_clause().types;
    if types.nodes.len() != 1 {
        return false;
    }
    let expr = types.nodes[0].as_expression_with_type_arguments().expression.clone();
    expr.kind == SyntaxKind::NullKeyword
}

fn opt_arc_eq(a: &Option<Arc<Node>>, b: &Option<Arc<Node>>) -> bool {
    match (a, b) {
        (Some(x), Some(y)) => Arc::ptr_eq(x, y),
        _ => false,
    }
}
