#[path = "r39k18_defs.rs"]
pub mod r39k18_defs;

#[path = "r40k21_defs.rs"]
pub mod r40k21_defs;

pub use r39k18_defs::{
    file_reference_from_ast, file_reference_to_ast, source_file_is_js, R39K18EmitResolverExt,
};

use super::m3n::*;
use super::m3n_5::r33k8_defs::{
    syntax_list_children, DeclarationEmitHost, FileReference, NodeId,
};
use super::m3n_5::{create_diagnostic_for_node, SymbolTrackerImpl, SymbolTrackerSharedState};
use super::m4e_2::{
    can_produce_diagnostics, is_declaration_and_not_visible, is_enclosing_declaration,
    is_scope_marker, needs_scope_marker,
};
use crate::mig::m3m_2::create_get_symbol_accessibility_diagnostic_for_node;
use crate::mig::m4e::{R37K1DataExt, R38K1NodeExt, R38K1NodeVisitorExt, R39K01EmitResolverExt};
use crate::printer::EmitContext;
use crate::printer::NodeFactory;
use tsox_frontend::ast::node_flags::{NodeFlags};
use tsox_frontend::ast::{is_declaration, is_function_like, is_global_scope_augmentation};
use tsox_core::diagnostics::messages_generated as diag_msgs;
use tsox_frontend::ast::mig::w7a::is_expando_property_declaration;
use tsox_frontend::scanner::CommentRange;
use crate::mig::m4k_2::Transformer;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tsox_checker::checker::mig::m2d::EmitResolver;
use tsox_checker::checker::types::SymbolAccessibilityResult;
use tsox_core::core::compiler_options::CompilerOptions;
use tsox_frontend::ast::diagnostic::Diagnostic;
use tsox_frontend::ast::mig::m3f::get_node_id;
use tsox_frontend::ast::mig::m3f_4::is_dynamic_name;
use tsox_frontend::ast::mig::m3g::is_late_visibility_painted_statement;
use tsox_frontend::ast::mig::m3h::try_get_text_of_property_name;
use tsox_frontend::ast::mig::w7a::is_external_module_indicator;
use tsox_frontend::ast::node_data_generated::*;
use tsox_frontend::ast::node_node::Node;
use tsox_frontend::ast::visitor::NodeVisitor;
use tsox_frontend::ast::{NodeList, SourceFile, SyntaxKind};

pub struct ReferencedFilePair {
    pub file: Arc<SourceFile>,
    pub reference: FileReference,
}

pub trait OutputPaths {
    fn declaration_file_path(&self) -> String;
    fn js_file_path(&self) -> String;
}

#[derive(Clone)]
pub struct ThisPropertyAssignmentKey {
    pub name: Option<String>,
    pub node: Option<Arc<Node>>,
    pub is_static: bool,
    pub is_private: bool,
}

impl PartialEq for ThisPropertyAssignmentKey {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.is_static == other.is_static
            && self.is_private == other.is_private
            && match (&self.node, &other.node) {
                (Some(a), Some(b)) => Arc::ptr_eq(a, b),
                (None, None) => true,
                _ => false,
            }
    }
}

impl Eq for ThisPropertyAssignmentKey {}

impl std::hash::Hash for ThisPropertyAssignmentKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.is_static.hash(state);
        self.is_private.hash(state);
        if let Some(node) = &self.node {
            (Arc::as_ptr(node) as usize).hash(state);
        }
    }
}

pub fn get_this_property_assignment_key(
    name: Option<&Arc<Node>>,
    node: &Arc<Node>,
    is_static: bool,
) -> ThisPropertyAssignmentKey {
    let is_private = name.map(|n| is_private_identifier(n)).unwrap_or(false);
    if let Some(name) = name {
        if !is_dynamic_name(name) {
            if let Some(name_text) = try_get_text_of_property_name(name) {
                return ThisPropertyAssignmentKey {
                    name: Some(name_text),
                    node: None,
                    is_static,
                    is_private,
                };
            }
        }
    }
    ThisPropertyAssignmentKey {
        name: None,
        node: Some(Arc::clone(node)),
        is_static,
        is_private,
    }
}

pub struct DeclarationTransformer {
    pub transformer: Transformer,
    pub host: DeclarationEmitHost,
    pub compiler_options: CompilerOptions,
    pub tracker: SymbolTrackerImpl,
    pub state: SymbolTrackerSharedState,
    pub resolver: EmitResolver,
    pub declaration_file_path: String,
    pub declaration_map_path: String,

    pub needs_declare: bool,
    pub needs_scope_fix_marker: bool,
    pub result_has_scope_marker: bool,
    pub enclosing_declaration: Option<Arc<Node>>,
    pub result_has_external_module_indicator: bool,
    pub suppress_new_diagnostic_contexts: bool,
    pub witnessed_cjs_exports: HashSet<String>,
    pub late_statement_replacement_map: HashMap<NodeId, Option<Arc<Node>>>,
    pub expando_hosts: HashMap<NodeId, Arc<Node>>,
    pub expando_members: HashMap<NodeId, Vec<Arc<Node>>>,
    pub deferred_expando_assignments: HashMap<NodeId, Vec<Arc<Node>>>,
    pub seen_properties: HashSet<ThisPropertyAssignmentKey>,
    pub this_property_assignments_collected: Vec<Arc<Node>>,
    pub raw_referenced_files: Vec<ReferencedFilePair>,
    pub raw_type_reference_directives: Vec<FileReference>,
    pub raw_lib_reference_directives: Vec<FileReference>,
    pub binding_name_visitor: NodeVisitor,
    pub expression_visitor: NodeVisitor,
    pub cjs_export_assignment_visitor: NodeVisitor,
    pub export_stripping_visitor: NodeVisitor,
    pub this_property_visitor: NodeVisitor,
    pub declare_stripping_visitor: NodeVisitor,

    pub cjs_export_assignment: Option<Arc<Node>>,
    pub cjs_export_members: Vec<Arc<Node>>,
    pub cjs_export_assignment_name: Option<Arc<Node>>,
    pub in_class_expression_declaration: bool,
}

pub fn new_declaration_transformer(
    host: DeclarationEmitHost,
    context: &EmitContext,
    compiler_options: CompilerOptions,
    declaration_file_path: String,
    declaration_map_path: String,
) -> DeclarationTransformer {
    let isolated_declarations = compiler_options.isolated_declarations.is_true();
    let strip_internal = compiler_options.strip_internal.is_true();
    let state = SymbolTrackerSharedState {
        isolated_declarations,
        strip_internal,
        ..SymbolTrackerSharedState::empty()
    };
    let tracker = super::m3n_5::new_symbol_tracker(host.clone(), host.get_emit_resolver(), state);
    let resolver = host.get_emit_resolver();
    let mut tx = DeclarationTransformer {
        transformer: Transformer::new(declaration_transformer_visit, Some(context.clone())),
        host,
        compiler_options,
        tracker,
        state: SymbolTrackerSharedState {
            isolated_declarations,
            strip_internal,
            ..SymbolTrackerSharedState::empty()
        },
        resolver,
        declaration_file_path,
        declaration_map_path,
        needs_declare: false, needs_scope_fix_marker: false, result_has_scope_marker: false,
        enclosing_declaration: None, result_has_external_module_indicator: false,
        suppress_new_diagnostic_contexts: false, witnessed_cjs_exports: HashSet::new(),
        late_statement_replacement_map: HashMap::new(), expando_hosts: HashMap::new(),
        expando_members: HashMap::new(), deferred_expando_assignments: HashMap::new(),
        seen_properties: HashSet::new(), this_property_assignments_collected: Vec::new(),
        raw_referenced_files: Vec::new(), raw_type_reference_directives: Vec::new(),
        raw_lib_reference_directives: Vec::new(),
        binding_name_visitor: NodeVisitor::default(),
        expression_visitor: NodeVisitor::default(),
        export_stripping_visitor: NodeVisitor::default(),
        this_property_visitor: NodeVisitor::default(),
        cjs_export_assignment_visitor: NodeVisitor::default(),
        declare_stripping_visitor: NodeVisitor::default(),
        cjs_export_assignment: None,
        cjs_export_members: Vec::new(),
        cjs_export_assignment_name: None,
        in_class_expression_declaration: false,
    };
    tx.install_expando_function_error_reporter();
    tx
}

impl DeclarationTransformer {
    pub fn get_diagnostics(&self) -> Vec<Diagnostic> {
        let mut diagnostics = self.state.diagnostics.clone();
        diagnostics.extend(self.tracker.state.diagnostics.iter().cloned());
        diagnostics
    }

    pub fn install_expando_function_error_reporter(&mut self) {
        let isolated_declarations = self.tracker.state.isolated_declarations;
        let host = self.host.clone();
        self.tracker.state.report_expando_function_errors =
            Some(Box::new(move |node: &Arc<Node>| {
                if !isolated_declarations {
                    return Vec::new();
                }
                let resolver = host.get_emit_resolver();
                let props = resolver.get_properties_of_container_function_unsafe(Some(node));
                let mut diags: Vec<Diagnostic> = Vec::new();
                for p in props {
                    let Some(value_declaration) = &p.value_declaration else {
                        continue;
                    };
                    if !is_expando_property_declaration(Some(value_declaration)) {
                        continue;
                    }
                    let mut error_target = Arc::clone(value_declaration);
                    if let NodeData::BinaryExpression(d) = &error_target.data {
                        error_target = Arc::clone(&d.left);
                    }
                    diags.push(create_diagnostic_for_node(
                        &error_target,
                        Some(&diag_msgs::ASSIGNING_PROPERTIES_TO_FUNCTIONS_WITHOUT_DECLARING_THEM_IS_NOT_SUPPORTED_WITH_ISOLATEDDECLARATIONS_ADD_AN_EXPLICIT_DECLARATION_FOR_THE_PROPERTIES_ASSIGNED_TO_THIS_FUNCTION),
                        &[],
                    ));
                }
                diags
            }));
    }

    pub fn collect_file_references(&mut self, source_file: &Arc<SourceFile>) {
        for reference in &source_file.referenced_files {
            self.raw_referenced_files.push(ReferencedFilePair {
                file: Arc::clone(source_file),
                reference: file_reference_from_ast(reference),
            });
        }
        self.raw_type_reference_directives
            .extend(source_file.type_reference_directives.iter().map(file_reference_from_ast));
        self.raw_lib_reference_directives
            .extend(source_file.lib_reference_directives.iter().map(file_reference_from_ast));
    }

    pub fn append_cjs_exports(
        &self,
        combined_statements: Arc<NodeList>,
    ) -> Arc<NodeList> {
        let mut result: Vec<Arc<Node>> = Vec::new();
        if let Some(cjs_export_assignment) = &self.cjs_export_assignment {
            result.push(Arc::clone(cjs_export_assignment));
        }
        result.extend(self.cjs_export_members.iter().cloned());
        result.extend(combined_statements.nodes.iter().cloned());
        let statement_nodes = flatten_syntax_lists(&result);
        if statement_nodes.len() != combined_statements.nodes.len() {
            return self.factory().new_node_list(statement_nodes);
        }
        combined_statements
    }

    pub fn transform_and_replace_late_painted_statements(
        &mut self,
        statements: &Arc<NodeList>,
    ) -> Arc<NodeList> {
        loop {
            if self.state.late_marked_statements.is_empty() {
                break;
            }
            let next = Arc::clone(&self.state.late_marked_statements[0]);
            self.state.late_marked_statements.remove(0);

            let save_needs_declare = self.needs_declare;
            self.needs_declare = next
                .parent()
                .map(|p| is_source_file(&p))
                .unwrap_or(false);

            let result = self.transform_top_level_declaration(Arc::clone(&next));

            self.needs_declare = save_needs_declare;
            let original = self.emit_context().most_original(&next);
            let id = get_node_id(&original);
            self.late_statement_replacement_map.insert(id, result);
        }

        let mut results: Vec<Arc<Node>> = Vec::with_capacity(statements.nodes.len());
        for statement in &statements.nodes {
            if !is_late_visibility_painted_statement(statement) {
                results.push(Arc::clone(statement));
                continue;
            }
            let original = self.emit_context().most_original(statement);
            let id = get_node_id(&original);
            let Some(replacement) = self.late_statement_replacement_map.get(&id) else {
                results.push(Arc::clone(statement));
                continue;
            };
            let Some(replacement) = replacement else {
                continue;
            };
            if replacement.kind == SyntaxKind::SyntaxList {
                if !self.needs_scope_fix_marker || !self.result_has_external_module_indicator {
                    for elem in syntax_list_children(replacement) {
                        if needs_scope_marker(&elem) {
                            self.needs_scope_fix_marker = true;
                        }
                        if statement.parent().map(|p| is_source_file(&p)).unwrap_or(false)
                            && is_external_module_indicator(&elem)
                        {
                            self.result_has_external_module_indicator = true;
                        }
                    }
                }
                results.extend(syntax_list_children(replacement));
            } else {
                if needs_scope_marker(replacement) {
                    self.needs_scope_fix_marker = true;
                }
                if statement.parent().map(|p| is_source_file(&p)).unwrap_or(false)
                    && is_external_module_indicator(replacement)
                {
                    self.result_has_external_module_indicator = true;
                }
                results.push(Arc::clone(replacement));
            }
        }
        self.factory().new_node_list(results)
    }

    pub fn transform_top_level_declaration(&mut self, input: Arc<Node>) -> Option<Arc<Node>> {
        if !self.state.late_marked_statements.is_empty() {
            self.state
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
            && is_declaration_and_not_visible(&self.emit_context(), &self.resolver, &input)
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
        let old_diag = self.state.get_symbol_accessibility_diagnostic.take();
        let old_name = self.state.error_name_node.clone();
        if can_produce_diagnostic {
            self.state.get_symbol_accessibility_diagnostic =
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
        self.state.get_symbol_accessibility_diagnostic = old_diag;
        self.needs_declare = save_needs_declare;
        self.state.error_name_node = old_name;
        result
    }

    pub fn transform_type_alias_declaration(&mut self, input: &Arc<Node>) -> Option<Arc<Node>> {
        self.needs_declare = false;
        let ta = input.as_type_alias_declaration();
        let type_parameters = self.visitor().visit_nodes(
            ta.type_parameters
                .clone()
                .unwrap_or_else(|| Arc::new(NodeList::new(Vec::new()))),
        );
        let type_node = self.visitor().visit(ta.type_node.clone());
        let modifiers = self.ensure_modifiers(input);
        let name = input.name().unwrap().clone();
        Some(self.factory().update_type_alias_declaration(
            input,
            modifiers,
            name,
            type_parameters,
            type_node,
        ))
    }

    pub fn transform_interface_declaration(&mut self, input: &Arc<Node>) -> Option<Arc<Node>> {
        let id = input.as_interface_declaration();
        let type_parameters = self.visitor().visit_nodes(
            id.type_parameters
                .clone()
                .unwrap_or_else(|| Arc::new(NodeList::new(Vec::new()))),
        );
        let heritage_clauses = self.visitor().visit_nodes(
            id.heritage_clauses
                .clone()
                .unwrap_or_else(|| Arc::new(NodeList::new(Vec::new()))),
        );
        let members = self.visitor().visit_nodes(id.members.clone());
        let modifiers = self.ensure_modifiers(input);
        let name = input.name().unwrap().clone();
        Some(self.factory().update_interface_declaration(
            input,
            modifiers,
            name,
            type_parameters,
            heritage_clauses,
            members,
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
                let mut late_statements =
                    self.transform_and_replace_late_painted_statements(&statements);
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
                        late_statements = self.export_stripping_visitor.visit_nodes(late_statements);
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

    pub fn get_referenced_files(&self, output_file_path: &str) -> Vec<FileReference> {
        super::m3n_9::get_referenced_files(self, output_file_path)
    }

    pub fn get_lib_references(&self) -> Vec<FileReference> {
        super::m3n_9::get_lib_references(self)
    }

    pub fn get_type_references(&self) -> Vec<FileReference> {
        super::m3n_9::get_type_references(self)
    }
}

fn declaration_transformer_visit(
    _transformer: &mut Transformer,
    node: Arc<Node>,
) -> Option<Arc<Node>> {
    Some(node)
}

pub fn throw_diagnostic(
    _result: &SymbolAccessibilityResult,
) -> Option<SymbolAccessibilityDiagnostic> {
    panic!("Diagnostic emitted without context")
}

pub fn node_or_syntax_list_children(node: &Arc<Node>) -> Vec<Arc<Node>> {
    if is_syntax_list(node) {
        syntax_list_children(node)
    } else {
        vec![Arc::clone(node)]
    }
}

pub fn flatten_syntax_lists(nodes: &[Arc<Node>]) -> Vec<Arc<Node>> {
    nodes.iter().flat_map(node_or_syntax_list_children).collect()
}

pub fn create_empty_exports(factory: &NodeFactory) -> Arc<Node> {
    let elements = factory.new_node_list(Vec::new());
    factory.new_export_declaration(
        None,
        false,
        &factory.new_named_exports(&elements),
        None,
        None,
    )
}

pub fn has_internal_annotation(comment_range: CommentRange, source_file: &SourceFile) -> bool {
    let text = &source_file.text[comment_range.pos..comment_range.end];
    text.contains("@internal")
}
