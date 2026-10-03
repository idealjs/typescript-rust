#![allow(unused_imports)]
#![allow(dead_code)]

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tsox_checker::binder::referenceresolver::ReferenceResolver;
use tsox_checker::checker::mig::m2d::EmitResolver;
use tsox_core::core::compiler_options::{CompilerOptions, ModuleKind};
use tsox_core::core::core::{append_if_unique, some};
use tsox_core::stringutil::compare_strings_case_sensitive;
use tsox_frontend::ast::Node;
use tsox_frontend::ast::NodeList;
use tsox_frontend::ast::SyntaxKind;
use tsox_frontend::ast::mig::m3e_4::get_namespace_declaration_node;
use tsox_frontend::ast::mig::m3f_4::is_default_import;
use tsox_frontend::ast::mig::m3g_3::module_export_name_is_default;
use tsox_frontend::ast::mig::w7a::is_external_module_import_equals_declaration;
use tsox_frontend::ast::mig::x6a::is_effective_external_module;
use tsox_frontend::ast::node_data_generated::*;
use tsox_frontend::ast::node_flags::ModifierFlags;
use tsox_frontend::ast::utilities::{has_syntactic_modifier, is_external_module, is_in_js_file};

use crate::mig::m4k::{EmitContext, NodeFactory};
pub use crate::mig::m4k::r39k02_defs::{R39K02EmitResolverExt, R39K02NodeExt};
use crate::mig::m4k_2::Transformer;
use crate::mig::m4m_2::is_local_name;
use crate::mig::m3m::TransformOptions;
use tsox_frontend::format::mig::m4o::EmitFlags;
use tsox_frontend::format::mig::m4o_2::EmitHelper;

use crate::mig::m4i_12::r36k17_defs::{
    NodeAsR36k17Ext, TOKEN_FLAGS_NONE,
};
use crate::mig::m4j::r36k3_defs::{R36K3NodeAccessExt, R36K3NodeFactoryExt};

pub struct ExternalModuleInfo {
    pub external_imports: Vec<Arc<Node>>,
    pub export_specifiers: HashMap<String, Vec<Arc<Node>>>,
    pub exported_bindings: Vec<(Arc<Node>, Arc<Node>)>,
    pub exported_names: Vec<Arc<Node>>,
    pub exported_functions: Vec<Arc<Node>>,
    pub export_equals: Option<Arc<Node>>,
    pub has_export_stars_to_export_values: bool,
}

pub struct ExternalModuleInfoCollector<'a> {
    pub source_file: &'a Arc<Node>,
    pub compiler_options: &'a CompilerOptions,
    pub emit_context: EmitContext,
    pub resolver: Arc<dyn ReferenceResolver>,
    pub unique_exports: HashSet<String>,
    pub has_export_default: bool,
    pub output: ExternalModuleInfo,
}

pub fn collect_external_module_info(
    source_file: &Arc<Node>,
    compiler_options: &CompilerOptions,
    emit_context: EmitContext,
    resolver: Arc<dyn ReferenceResolver>,
) -> ExternalModuleInfo { ::tsox_core::fntrace::enter("collect_external_module_info"); 
    let mut c = ExternalModuleInfoCollector {
        source_file,
        compiler_options,
        emit_context,
        resolver,
        unique_exports: HashSet::new(),
        has_export_default: false,
        output: ExternalModuleInfo {
            external_imports: Vec::new(),
            export_specifiers: HashMap::new(),
            exported_bindings: Vec::new(),
            exported_names: Vec::new(),
            exported_functions: Vec::new(),
            export_equals: None,
            has_export_stars_to_export_values: false,
        },
    };
    c.collect()
}

impl<'a> ExternalModuleInfoCollector<'a> {
    fn factory(&self) -> NodeFactory<'_> { ::tsox_core::fntrace::enter("factory"); 
        NodeFactory::new(&self.emit_context)
    }

    pub fn collect(&mut self) -> ExternalModuleInfo { ::tsox_core::fntrace::enter("collect"); 
        let mut has_import_star = false;
        let mut has_import_default = false;
        let statements = self.source_file.as_source_file_data().statements.nodes.clone();
        for node in statements {
            if is_not_emitted_statement(&node) {
                let original = self.emit_context.most_original(&node);
                if is_export_assignment(&original) {
                    if original.as_export_assignment().is_export_equals
                        && self.output.export_equals.is_none()
                    {
                        self.output.export_equals = Some(original.clone());
                    }
                }
                continue;
            }
            match node.kind {
                SyntaxKind::ImportDeclaration => {
                    self.add_external_import(&node);
                    if !has_import_star && get_import_needs_import_star_helper(&node) {
                        has_import_star = true;
                    }
                    if !has_import_default && get_import_needs_import_default_helper(&node) {
                        has_import_default = true;
                    }
                }
                SyntaxKind::ImportEqualsDeclaration => {
                    if is_external_module_reference(
                        &node.as_import_equals_declaration().module_reference,
                    ) {
                        self.add_external_import(&node);
                    }
                }
                SyntaxKind::ExportDeclaration => {
                    let export_decl = node.as_export_declaration();
                    if export_decl.module_specifier.is_some() {
                        self.add_external_import(&node);
                        if export_decl.export_clause.is_none() {
                            self.output.has_export_stars_to_export_values = true;
                        } else if is_named_exports(export_decl.export_clause.as_ref().unwrap()) {
                            self.add_exported_names_for_export_declaration(&node);
                            if !has_import_default {
                                has_import_default = contains_default_reference(
                                    export_decl.export_clause.as_ref().unwrap(),
                                );
                            }
                        } else {
                            let name = export_decl
                                .export_clause
                                .as_ref()
                                .unwrap()
                                .as_namespace_export()
                                .name
                                .clone();
                            let name_text = name.text().to_string();
                            if self.add_unique_export(&name_text) {
                                self.add_exported_binding(&node, &name);
                                self.add_exported_name(name);
                            }
                            has_import_star = true;
                        }
                    } else {
                        self.add_exported_names_for_export_declaration(&node);
                    }
                }
                SyntaxKind::ExportAssignment => {
                    if node.as_export_assignment().is_export_equals
                        && self.output.export_equals.is_none()
                    {
                        self.output.export_equals = Some(node.clone());
                    }
                }
                SyntaxKind::VariableStatement => {
                    if has_syntactic_modifier(&node, ModifierFlags::Export) {
                        for decl in node
                            .as_variable_statement()
                            .declaration_list
                            .as_variable_declaration_list()
                            .declarations
                            .nodes
                            .clone()
                        {
                            self.collect_exported_variable_info(&decl);
                        }
                    }
                }
                SyntaxKind::FunctionDeclaration => {
                    if has_syntactic_modifier(&node, ModifierFlags::Export) {
                        self.add_exported_function_declaration(
                            &node,
                            None,
                            has_syntactic_modifier(&node, ModifierFlags::Default),
                        );
                    }
                }
                SyntaxKind::ClassDeclaration => {
                    if has_syntactic_modifier(&node, ModifierFlags::Export) {
                        if has_syntactic_modifier(&node, ModifierFlags::Default) {
                            if !self.has_export_default {
                                let mut name = node.name().cloned();
                                if name.is_none() {
                                    name = Some(
                                        self.emit_context
                                            .factory()
                                            .generated_name_node(&self.emit_context
                                                .factory()
                                                .new_generated_name_for_node(&node)),
                                    );
                                }
                                self.add_exported_binding(&node, name.as_ref().unwrap());
                                self.has_export_default = true;
                            }
                        } else {
                            let name = node.name().cloned();
                            if let Some(name) = name {
                                if self.add_unique_export(&name.text().to_string()) {
                                    self.add_exported_binding(&node, &name);
                                    self.add_exported_name(name);
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        std::mem::replace(
            &mut self.output,
            ExternalModuleInfo {
                external_imports: Vec::new(),
                export_specifiers: HashMap::new(),
                exported_bindings: Vec::new(),
                exported_names: Vec::new(),
                exported_functions: Vec::new(),
                export_equals: None,
                has_export_stars_to_export_values: false,
            },
        )
    }

    pub fn add_unique_export(&mut self, name: &str) -> bool { ::tsox_core::fntrace::enter("add_unique_export"); 
        if !self.unique_exports.contains(name) {
            self.unique_exports.insert(name.to_string());
            return true;
        }
        false
    }

    pub fn add_exported_binding(&mut self, decl: &Arc<Node>, name: &Arc<Node>) { ::tsox_core::fntrace::enter("add_exported_binding"); 
        let original = self.emit_context.most_original(decl);
        self.output
            .exported_bindings
            .push((original, name.clone()));
    }

    pub fn add_external_import(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("add_external_import"); 
        self.output.external_imports.push(node.clone());
    }

    pub fn add_exported_name(&mut self, name: Arc<Node>) { ::tsox_core::fntrace::enter("add_exported_name"); 
        self.output.exported_names.push(name);
    }

    pub fn add_exported_names_for_export_declaration(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("add_exported_names_for_export_declaration"); 
        let export_decl = node.as_export_declaration();
        let elements = export_decl
            .export_clause
            .as_ref()
            .unwrap()
            .elements_list_r39k02()
            .nodes
            .clone();
        for specifier in elements {
            let specifier_name_text = specifier.name().unwrap().text().to_string();
            if self.add_unique_export(&specifier_name_text) {
                let name = specifier.property_name_or_name();
                if name.kind != SyntaxKind::StringLiteral {
                    if export_decl.module_specifier.is_none() {
                        self.output
                            .export_specifiers
                            .entry(name.text().to_string())
                            .or_default()
                            .push(specifier.clone());
                    }

                    let original = self.emit_context.most_original(&name);
                    let mut decl = self
                        .resolver
                        .get_referenced_import_declaration(&original);
                    if decl.is_none() {
                        decl = self
                            .resolver
                            .get_referenced_value_declaration(&original);
                    }
                    if let Some(decl) = decl {
                        if decl.kind == SyntaxKind::FunctionDeclaration {
                            self.unique_exports.remove(&specifier_name_text);
                            self.add_exported_function_declaration(
                                &decl,
                                specifier.name(),
                                module_export_name_is_default(specifier.name().unwrap()),
                            );
                            continue;
                        }
                        self.add_exported_binding(&decl, specifier.name().unwrap());
                    }
                }

                self.add_exported_name(specifier.name().unwrap().clone());
            }
        }
    }

    pub fn add_exported_function_declaration(
        &mut self,
        node: &Arc<Node>,
        name: Option<&Arc<Node>>,
        is_default: bool,
    ) { ::tsox_core::fntrace::enter("add_exported_function_declaration"); 
        let original = self.emit_context.most_original(node);
        self.output.exported_functions.push(original);
        if is_default {
            if !self.has_export_default {
                let name = match name {
                    Some(name) => name.clone(),
                    None => self
                        .emit_context
                        .factory()
                        .generated_name_node(&self.emit_context.factory().new_generated_name_for_node(node)),
                };
                self.add_exported_binding(node, &name);
                self.has_export_default = true;
            }
        } else {
            let name = match name {
                Some(name) => name.clone(),
                None => node.name().unwrap().clone(),
            };
            let name_text = name.text().to_string();
            if self.add_unique_export(&name_text) {
                self.add_exported_binding(node, &name);
            }
        }
    }

    pub fn collect_exported_variable_info(&mut self, decl: &Arc<Node>) { ::tsox_core::fntrace::enter("collect_exported_variable_info"); 
        let name = decl.name().unwrap().clone();
        if is_binding_pattern(&name) {
            for element in name.elements_list_r39k02().nodes.clone() {
                if element.as_binding_element().name.is_some() {
                    self.collect_exported_variable_info(&element);
                }
            }
        } else if !self.emit_context.has_auto_generate_info(&name) {
            let text = name.text().to_string();
            if self.add_unique_export(&text) {
                self.add_exported_name(name.clone());
                if is_local_name(&self.emit_context, &name) {
                    self.add_exported_binding(decl, &name);
                }
            }
        }
    }
}

pub const EXTERNAL_HELPERS_MODULE_NAME_TEXT: &str = "tslib";

pub fn create_external_helpers_import_declaration_if_needed(
    mut emit_context: EmitContext,
    source_file: &Arc<Node>,
    compiler_options: &CompilerOptions,
    file_module_kind: ModuleKind,
    has_export_stars_to_export_values: bool,
    has_import_star: bool,
    has_import_default: bool,
) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("create_external_helpers_import_declaration_if_needed"); 
    if compiler_options.import_helpers.is_true()
        && source_file.is_effective_external_module_node(compiler_options)
    {
        let module_kind = compiler_options.get_emit_module_kind();
        let helpers = get_imported_helpers(&emit_context, source_file);
        if file_module_kind == ModuleKind::CommonJS
            || file_module_kind == ModuleKind::None && module_kind == ModuleKind::CommonJS
        {
            let external_helpers_module_name = get_or_create_external_helpers_module_name_if_needed(
                emit_context.clone(),
                source_file,
                compiler_options,
                &helpers,
                has_export_stars_to_export_values,
                has_import_star || has_import_default,
                file_module_kind,
            );
            if let Some(external_helpers_module_name) = external_helpers_module_name {
                let external_helpers_import_declaration =
                    emit_context.factory().new_import_equals_declaration(
                        None,
                        false,
                        external_helpers_module_name,
                        emit_context.factory().new_external_module_reference(
                            emit_context.factory().new_string_literal(
                                EXTERNAL_HELPERS_MODULE_NAME_TEXT,
                                TOKEN_FLAGS_NONE,
                            ),
                        ),
                    );
                emit_context.add_emit_flags(
                    &external_helpers_import_declaration,
                    EmitFlags::CUSTOM_PROLOGUE,
                );
                return Some(external_helpers_import_declaration);
            }
        } else {
            let mut helper_names: Vec<String> = Vec::new();
            for helper in &helpers {
                let import_name = &helper.import_name;
                if !import_name.is_empty() {
                    helper_names = append_if_unique(&helper_names, import_name);
                }
            }
            if !helper_names.is_empty() {
                helper_names.sort_by(|a, b| compare_strings_case_sensitive(a, b).cmp(&0));
                let import_specifiers: Vec<Arc<Node>> = helper_names
                    .iter()
                    .map(|name| {
                        if emit_context.is_file_level_unique_name(source_file, name, None) {
                            emit_context.factory().new_import_specifier(
                                false,
                                None,
                                emit_context.factory().new_identifier(name),
                            )
                        } else {
                            emit_context.factory().new_import_specifier(
                                false,
                                Some(emit_context.factory().new_identifier(name)),
                                emit_context.factory().new_unscoped_helper_name(name),
                            )
                        }
                    })
                    .collect();
                let named_bindings = emit_context
                    .factory()
                    .new_named_imports(emit_context.factory().new_node_list(import_specifiers));
                let parse_node = emit_context.most_original(source_file);
                emit_context.add_emit_flags(&parse_node, EmitFlags::EXTERNAL_HELPERS);

                let external_helpers_import_declaration =
                    emit_context.factory().new_import_declaration(
                        None,
                        Some(emit_context.factory().new_import_clause(
                            SyntaxKind::Unknown,
                            None,
                            Some(named_bindings),
                        )),
                        emit_context.factory().new_string_literal(
                            EXTERNAL_HELPERS_MODULE_NAME_TEXT,
                            TOKEN_FLAGS_NONE,
                        ),
                        None,
                    );

                emit_context.add_emit_flags(
                    &external_helpers_import_declaration,
                    EmitFlags::CUSTOM_PROLOGUE,
                );
                return Some(external_helpers_import_declaration);
            }
        }
    }
    None
}

pub fn get_imported_helpers(
    emit_context: &EmitContext,
    source_file: &Arc<Node>,
) -> Vec<Arc<EmitHelper>> { ::tsox_core::fntrace::enter("get_imported_helpers"); 
    let mut helpers: Vec<Arc<EmitHelper>> = Vec::new();
    for helper in emit_context.get_emit_helpers(source_file) {
        if !helper.scoped {
            helpers.push(helper);
        }
    }
    helpers
}

pub fn get_or_create_external_helpers_module_name_if_needed(
    mut emit_context: EmitContext,
    node: &Arc<Node>,
    compiler_options: &CompilerOptions,
    helpers: &[Arc<EmitHelper>],
    has_export_stars_to_export_values: bool,
    has_import_star_or_import_default: bool,
    file_module_kind: ModuleKind,
) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("get_or_create_external_helpers_module_name_if_needed"); 
    if let Some(external_helpers_module_name) =
        emit_context.get_external_helpers_module_name(node)
    {
        return Some(external_helpers_module_name);
    }

    let create = !helpers.is_empty()
        || (has_export_stars_to_export_values || has_import_star_or_import_default)
            && file_module_kind < ModuleKind::System;

    if create {
        let external_helpers_module_name = emit_context.factory().generated_name_node(
            &emit_context
                .factory()
                .new_unique_name(EXTERNAL_HELPERS_MODULE_NAME_TEXT),
        );
        emit_context
            .set_external_helpers_module_name(node, &external_helpers_module_name);
        return Some(external_helpers_module_name);
    }

    None
}

pub fn is_named_default_reference(e: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_named_default_reference"); 
    module_export_name_is_default(&e.property_name_or_name())
}

pub fn contains_default_reference(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("contains_default_reference"); 
    is_named_imports(node)
        || is_named_exports(node) && some(&node.elements_list_r39k02().nodes, is_named_default_reference)
}

pub fn get_export_needs_import_star_helper(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("get_export_needs_import_star_helper"); 
    get_namespace_declaration_node(node).is_some()
}

pub fn get_import_needs_import_star_helper(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("get_import_needs_import_star_helper"); 
    if get_namespace_declaration_node(node).is_some() {
        return true;
    }
    let import_clause = node.as_import_declaration().import_clause.clone();
    if import_clause.is_none() {
        return false;
    }
    let bindings = import_clause.unwrap().as_import_clause().named_bindings.clone();
    if bindings.is_none() {
        return false;
    }
    let bindings = bindings.unwrap();
    if !is_named_imports(&bindings) {
        return false;
    }
    let named_imports = bindings.as_named_imports();
    let elements = named_imports.elements.nodes.clone();
    let mut default_ref_count = 0;
    for binding in &elements {
        if is_named_default_reference(binding) {
            default_ref_count += 1;
        }
    }
    (default_ref_count > 0 && default_ref_count != elements.len())
        || ((elements.len() - default_ref_count) != 0 && is_default_import(node))
}

pub fn get_import_needs_import_default_helper(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("get_import_needs_import_default_helper"); 
    !get_import_needs_import_star_helper(node)
        && (is_default_import(node)
            || (node.as_import_declaration().import_clause.is_some()
                && is_named_imports(
                    node.as_import_declaration()
                        .import_clause
                        .as_ref()
                        .unwrap()
                        .as_import_clause()
                        .named_bindings
                        .as_ref()
                        .unwrap(),
                )
                && contains_default_reference(
                    node.as_import_declaration()
                        .import_clause
                        .as_ref()
                        .unwrap()
                        .as_import_clause()
                        .named_bindings
                        .as_ref()
                        .unwrap(),
                )))
}

pub struct ImportElisionTransformer<'a> {
    pub emit_context: &'a EmitContext,
    pub compiler_options: &'a CompilerOptions,
    pub current_source_file: Option<Arc<Node>>,
    pub emit_resolver: &'a EmitResolver,
}

pub fn new_import_elision_transformer(opt: &TransformOptions) -> Transformer { ::tsox_core::fntrace::enter("new_import_elision_transformer"); 
    let compiler_options = opt.compiler_options;
    let emit_context = opt.context;
    if compiler_options.verbatim_module_syntax.is_true() {
        panic!("ImportElisionTransformer should not be used with VerbatimModuleSyntax");
    }
    let _tx = ImportElisionTransformer {
        emit_context,
        compiler_options,
        current_source_file: None,
        emit_resolver: &opt.emit_resolver,
    };
    Transformer::new(import_elision_transformer_visit, Some(emit_context.clone()))
}

fn import_elision_transformer_visit(
    _transformer: &mut Transformer,
    node: Arc<Node>,
) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("import_elision_transformer_visit"); 
    Some(node)
}

impl<'a> ImportElisionTransformer<'a> {
    fn visitor(&self) -> crate::mig::m4k::Visitor { ::tsox_core::fntrace::enter("visitor"); 
        crate::mig::m4k::Visitor
    }

    fn factory(&self) -> NodeFactory<'_> { ::tsox_core::fntrace::enter("factory"); 
        NodeFactory::new(self.emit_context)
    }

    fn emit_context(&self) -> &'a EmitContext { ::tsox_core::fntrace::enter("emit_context"); 
        self.emit_context
    }

    fn visit(&mut self, node: Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("visit"); 
        if is_source_file(&node) {
            let most_original = self.emit_context().most_original(&node);
            self.emit_resolver
                .mark_linked_references_recursively_r39k02(&most_original);
        }

        match node.kind {
            SyntaxKind::ImportEqualsDeclaration => {
                if is_external_module_import_equals_declaration(&node) {
                    if !self.should_emit_alias_declaration(&node) {
                        return None;
                    }
                } else if !self.should_emit_import_equals_declaration(&node) {
                    return None;
                }
                self.visitor().visit_each_child(node)
            }
            SyntaxKind::ImportDeclaration => {
                let import_decl = node.as_import_declaration();
                if import_decl.import_clause.is_some() {
                    let import_clause = self
                        .visitor()
                        .visit_node(import_decl.import_clause.clone());
                    if import_clause.is_none() {
                        return None;
                    }
                    Some(self.factory().update_import_declaration(
                        &node,
                        node.modifiers().cloned(),
                        import_clause,
                        import_decl.module_specifier.clone(),
                        self.visitor().visit_node(import_decl.attributes.clone()),
                    ))
                } else {
                    self.visitor().visit_each_child(node)
                }
            }
            SyntaxKind::ImportClause => {
                let import_clause = node.as_import_clause();
                let name = if self.should_emit_alias_declaration(&node) {
                    import_clause.name.clone()
                } else {
                    None
                };
                let named_bindings = self
                    .visitor()
                    .visit_node(import_clause.named_bindings.clone());
                if name.is_none() && named_bindings.is_none() {
                    return None;
                }
                Some(self.factory().update_import_clause(
                    &node,
                    import_clause.phase_modifier,
                    name,
                    named_bindings,
                ))
            }
            SyntaxKind::NamespaceImport => {
                if !self.should_emit_alias_declaration(&node) {
                    return None;
                }
                Some(node)
            }
            SyntaxKind::NamedImports => {
                let named_imports = node.as_named_imports();
                let elements = self
                    .visitor()
                    .visit_nodes(named_imports.elements.nodes.clone());
                if elements.is_empty() {
                    return None;
                }
                let mut elements_list = NodeList::new(elements);
                elements_list.loc = named_imports.elements.loc;
                Some(self.factory().update_named_imports(&node, &elements_list))
            }
            SyntaxKind::ImportSpecifier => {
                if !self.should_emit_alias_declaration(&node) {
                    return None;
                }
                Some(node)
            }
            SyntaxKind::ExportAssignment => {
                if !self.compiler_options.verbatim_module_syntax.is_true()
                    && !self.is_value_alias_declaration(&node)
                {
                    return None;
                }
                self.visitor().visit_each_child(node)
            }
            SyntaxKind::ExportDeclaration => {
                let export_decl = node.as_export_declaration();
                let mut export_clause = None;
                if export_decl.export_clause.is_some() {
                    export_clause = self
                        .visitor()
                        .visit_node(export_decl.export_clause.clone());
                    if export_clause.is_none() {
                        return None;
                    }
                }
                Some(self.factory().update_export_declaration(
                    &node,
                    None,
                    false,
                    export_clause,
                    self.visitor()
                        .visit_node(export_decl.module_specifier.clone()),
                    self.visitor().visit_node(export_decl.attributes.clone()),
                ))
            }
            SyntaxKind::NamedExports => {
                let named_exports = node.as_named_exports();
                let elements = self
                    .visitor()
                    .visit_nodes(named_exports.elements.nodes.clone());
                if elements.is_empty() {
                    return None;
                }
                let mut elements_list = NodeList::new(elements);
                elements_list.loc = named_exports.elements.loc;
                Some(self.factory().update_named_exports(&node, &elements_list))
            }
            SyntaxKind::ExportSpecifier => {
                if !self.is_value_alias_declaration(&node) {
                    return None;
                }
                Some(node)
            }
            SyntaxKind::SourceFile => {
                let saved_current_source_file = self.current_source_file.take();
                self.current_source_file = Some(node.clone());
                let result = self.visitor().visit_each_child(node);
                self.current_source_file = saved_current_source_file;
                result
            }
            SyntaxKind::ModuleDeclaration | SyntaxKind::ModuleBlock => {
                self.visitor().visit_each_child(node)
            }
            _ => Some(node),
        }
    }

    pub fn should_emit_alias_declaration(&self, node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("should_emit_alias_declaration"); 
        is_in_js_file(node) || self.is_referenced_alias_declaration(node)
    }

    pub fn should_emit_import_equals_declaration(&self, node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("should_emit_import_equals_declaration"); 
        let current_source_file = self.current_source_file.clone().unwrap();
        self.should_emit_alias_declaration(node)
            || (!current_source_file.is_external_module_node()
                && self.is_top_level_value_import_equals_with_entity_name(node))
    }

    pub fn is_referenced_alias_declaration(&self, node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_referenced_alias_declaration"); 
        match self.emit_context().parse_node(node) {
            None => true,
            Some(node) => self
                .emit_resolver
                .is_referenced_alias_declaration_r39k02(&node),
        }
    }

    pub fn is_value_alias_declaration(&self, node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_value_alias_declaration"); 
        match self.emit_context().parse_node(node) {
            None => true,
            Some(node) => self.emit_resolver.is_value_alias_declaration_r39k02(&node),
        }
    }

    pub fn is_top_level_value_import_equals_with_entity_name(&self, node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_top_level_value_import_equals_with_entity_name"); 
        match self.emit_context().parse_node(node) {
            None => false,
            Some(node) => self
                .emit_resolver
                .is_top_level_value_import_equals_with_entity_name_r39k02(&node),
        }
    }
}
