#![allow(unused_imports)]
#![allow(dead_code)]

use std::sync::Arc;
use tsox_checker::binder::referenceresolver::ReferenceResolver;
use tsox_checker::checker::mig::m2d::EmitResolver;
use tsox_core::core::compiler_options::{CompilerOptions, ModuleKind};
use tsox_core::core::compiler_options_kinds::JsxEmit;
use tsox_core::core::core::some;
use tsox_core::core::mig::m3j_2::should_rewrite_module_specifier;
use tsox_core::tspath::change_extension;
use tsox_frontend::ast::{Node, NodeList};
use tsox_frontend::ast::SyntaxKind;
use tsox_frontend::ast::dynamic_imports::is_require_call;
use tsox_frontend::ast::mig::w7a::{
    is_external_module_import_equals_declaration, is_external_module_indicator,
    is_export_namespace_as_default_declaration,
};
use tsox_frontend::ast::node_data_generated::*;
use tsox_frontend::ast::node_flags::{ModifierFlags, NodeFlags};
use tsox_frontend::ast::utilities::{
    has_syntactic_modifier, is_external_module, is_import_call, is_in_js_file,
    is_string_literal_like,
};
use tsox_frontend::ast::visitor::NodeVisitor;

use crate::mig::m4j_2::new_common_js_module_transformer;
use crate::mig::m4k::{
    CommonJSModuleTransformer, EmitContext, HasFileName, NodeFactory, Visitor,
};
use crate::mig::m4k_3::create_external_helpers_import_declaration_if_needed;
use crate::mig::m4m_2::single_or_many;
use crate::mig::m4m_5::is_simple_copiable_expression;

use crate::mig::m3m::TransformOptions;
use crate::mig::r33k6_shim::{get_output_extension, EmitHost, ResolveModuleNameResolutionHost};
use crate::printer::get_external_module_name;
use tsox_frontend::format::mig::m4o::EmitFlags;
use tsox_frontend::scanner::TOKEN_FLAGS_NONE;
use crate::printer::{AutoGenerateOptions, GeneratedIdentifierFlags};

#[path = "r37k8_defs.rs"]
pub mod r37k8_defs;

use r37k8_defs::{empty_has_file_name, NodeAsR37k8Ext};
use crate::mig::m4k::R39K02NodeExt;

pub struct ESModuleTransformer<'a> {
    pub emit_context: EmitContext,
    pub compiler_options: &'a CompilerOptions,
    pub resolver: Arc<dyn ReferenceResolver>,
    pub get_emit_module_format_of_file: Arc<dyn Fn(&dyn HasFileName) -> ModuleKind + Send + Sync>,
    pub current_source_file: Option<Arc<Node>>,
    pub import_require_statements: Option<ImportRequireStatements>,
    pub helper_name_substitutions: std::collections::HashMap<String, Arc<Node>>,
}

pub struct ImportRequireStatements {
    pub statements: Vec<Arc<Node>>,
    pub require_helper_name: Arc<Node>,
}

pub fn new_es_module_transformer(opts: &TransformOptions) -> Transformer { ::tsox_core::fntrace::enter("new_es_module_transformer"); 
    let _tx = ESModuleTransformer {
        emit_context: opts.context.clone(),
        compiler_options: opts.compiler_options,
        resolver: opts.resolver.clone(),
        get_emit_module_format_of_file: opts.get_emit_module_format_of_file.clone(),
        current_source_file: None,
        import_require_statements: None,
        helper_name_substitutions: std::collections::HashMap::new(),
    };
    Transformer::new(es_module_transformer_visit, Some(opts.context.clone()))
}

fn es_module_transformer_visit(
    _transformer: &mut Transformer,
    node: Arc<Node>,
) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("es_module_transformer_visit"); 
    Some(node)
}

impl<'a> ESModuleTransformer<'a> {
    fn factory(&self) -> NodeFactory<'_> { ::tsox_core::fntrace::enter("factory"); 
        NodeFactory::new(&self.emit_context)
    }

    fn visitor(&self) -> Visitor { ::tsox_core::fntrace::enter("visitor"); 
        Visitor
    }

    fn emit_context(&self) -> EmitContext { ::tsox_core::fntrace::enter("emit_context"); 
        self.emit_context.clone()
    }

    fn visit(&mut self, node: Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("visit"); 
        match node.kind {
            SyntaxKind::SourceFile => self.visit_source_file(node),
            SyntaxKind::ImportDeclaration => self.visit_import_declaration(node),
            SyntaxKind::ImportEqualsDeclaration => self.visit_import_equals_declaration(node),
            SyntaxKind::ExportAssignment => self.visit_export_assignment(node),
            SyntaxKind::ExportDeclaration => self.visit_export_declaration(node),
            SyntaxKind::CallExpression => self.visit_call_expression(node),
            _ => self.visitor().visit_each_child(node),
        }
    }

    pub fn visit_source_file(&mut self, node: Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("visit_source_file"); 
        if node.is_declaration_file_node()
            || !(node.is_external_module_node() || self.compiler_options.get_isolated_modules())
        {
            return Some(node);
        }

        self.current_source_file = Some(node.clone());
        self.import_require_statements = None;

        let result = self.visitor().visit_each_child(node.clone())?;
        self.emit_context().add_emit_helper(
            &result,
            &self.emit_context().read_emit_helpers(),
        );

        let external_helpers_import_declaration =
            create_external_helpers_import_declaration_if_needed(
                self.emit_context(),
                &result,
                self.compiler_options,
                (self.get_emit_module_format_of_file)(&empty_has_file_name()),
                false,
                false,
                false,
            );
        if external_helpers_import_declaration.is_some() || self.import_require_statements.is_some()
        {
            let (prologue, rest) = r37k8_defs::split_standard_prologue_r37k8(
                &result.as_source_file().statements.nodes,
            );
            let statements_loc = result.as_source_file().statements.loc;
            let (custom, rest) = r37k8_defs::split_custom_prologue_r37k8(&rest);
            let mut statements = prologue;
            statements.extend(custom);
            if let Some(external_helpers_import_declaration) = external_helpers_import_declaration
            {
                statements.push(
                    self.visitor()
                        .visit_node(external_helpers_import_declaration)?,
                );
            }
            if let Some(import_require) = &self.import_require_statements {
                statements.extend(import_require.statements.clone());
            }
            statements.extend(rest);
            let mut statement_list = NodeList::new(statements);
            statement_list.loc = statements_loc;
            return Some(
                self.factory()
                    .update_source_file(&result, Arc::new(statement_list)),
            );
        }

        if result.is_external_module_node()
            && self.compiler_options.get_emit_module_kind() != ModuleKind::Preserve
            && !some(&result.as_source_file().statements.nodes, |n| is_external_module_indicator(n))
        {
            let statements_loc = result.as_source_file().statements.loc;
            let mut statements = result.as_source_file().statements.nodes.clone();
            statements.push(create_empty_imports(&self.factory()));
            let mut statement_list = NodeList::new(statements);
            statement_list.loc = statements_loc;
            return Some(
                self.factory()
                    .update_source_file(&result, Arc::new(statement_list)),
            );
        }

        self.import_require_statements = None;
        self.current_source_file = None;
        Some(result)
    }

    pub fn visit_import_declaration(&mut self, node: Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("visit_import_declaration"); 
        if !self.compiler_options.rewrite_relative_import_extensions.is_true() {
            return Some(node);
        }
        let import_decl = node.as_import_declaration();
        let updated_module_specifier = rewrite_module_specifier(
            self.emit_context(),
            Some(&import_decl.module_specifier),
            self.compiler_options,
        )
        .unwrap_or_else(|| import_decl.module_specifier.clone());
        Some(self.factory().update_import_declaration(
            &node,
            None,
            import_decl
                .import_clause
                .clone()
                .map(|n| self.visitor().visit_node(n))
                .flatten(),
            updated_module_specifier,
            import_decl
                .attributes
                .clone()
                .map(|n| self.visitor().visit_node(n))
                .flatten(),
        ))
    }

    pub fn visit_import_equals_declaration(
        &mut self,
        node: Arc<Node>,
    ) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("visit_import_equals_declaration"); 
        if self.compiler_options.get_emit_module_kind() < ModuleKind::Node16 {
            return None;
        }

        if !is_external_module_import_equals_declaration(&node) {
            panic!("import= for internal module references should be handled in an earlier transformer.");
        }

        let require_call = self.create_require_call(&node);
        let var_statement = self.factory().new_variable_statement(
            None,
            &self.factory().new_variable_declaration_list(
                &self.factory().new_node_list(vec![self.factory().new_variable_declaration(
                    node.name().unwrap(),
                    None,
                    None,
                    require_call.as_ref(),
                )]),
                NodeFlags::Const,
            ),
        );
        self.emit_context().set_original(&var_statement, &node);
        self.emit_context()
            .assign_comment_and_source_map_ranges(&var_statement, &node);

        let mut statements: Vec<Arc<Node>> = Vec::new();
        statements.push(var_statement);
        statements = self.append_exports_of_import_equals_declaration(statements, &node);
        single_or_many(Some(&statements), &self.factory())
    }

    pub fn append_exports_of_import_equals_declaration(
        &mut self,
        mut statements: Vec<Arc<Node>>,
        node: &Arc<Node>,
    ) -> Vec<Arc<Node>> { ::tsox_core::fntrace::enter("append_exports_of_import_equals_declaration"); 
        if has_syntactic_modifier(node, ModifierFlags::Export) {
            let named_exports = self.factory().new_named_exports(&self.factory().new_node_list(vec![
                self.factory().new_export_specifier(
                    false,
                    None,
                    node.name().unwrap(),
                ),
            ]));
            statements.push(self.factory().new_export_declaration(
                None,
                false,
                &named_exports,
                None,
                None,
            ));
        }
        statements
    }

    pub fn visit_export_assignment(&mut self, node: Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("visit_export_assignment"); 
        let export_assignment = node.as_export_assignment();
        if !export_assignment.is_export_equals {
            return self.visitor().visit_each_child(node);
        }
        if self.compiler_options.get_emit_module_kind() != ModuleKind::Preserve {
            return None;
        }
        let statement = self.factory().new_expression_statement(
            &self.factory().new_assignment_expression(
                &self.factory().new_property_access_expression(
                    &self.factory().new_identifier("module"),
                    None,
                    &self.factory().new_identifier("exports"),
                    NodeFlags::empty(),
                ),
                &self
                    .visitor()
                    .visit_node(export_assignment.expression.clone())?,
            ),
        );
        self.emit_context().set_original(&statement, &node);
        Some(statement)
    }

    pub fn visit_export_declaration(&mut self, node: Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("visit_export_declaration"); 
        let export_decl = node.as_export_declaration();
        if export_decl.module_specifier.is_none() {
            return Some(node.clone());
        }

        let updated_module_specifier = rewrite_module_specifier(
            self.emit_context(),
            export_decl.module_specifier.as_ref(),
            self.compiler_options,
        );
        if self.compiler_options.module > ModuleKind::ES2015
            || export_decl.export_clause.is_none()
            || !is_namespace_export(export_decl.export_clause.as_ref().unwrap())
        {
            return Some(self.factory().update_export_declaration(
                &node,
                None,
                false,
                export_decl
                    .export_clause
                    .clone()
                    .map(|n| self.visitor().visit_node(n))
                    .flatten(),
                updated_module_specifier,
                export_decl
                    .attributes
                    .clone()
                    .map(|n| self.visitor().visit_node(n))
                    .flatten(),
            ));
        }

        let old_identifier = export_decl
            .export_clause
            .clone()
            .unwrap()
            .name()
            .unwrap()
            .clone();
        let synth_name = self.factory().generated_name_node(
            &self.factory().new_generated_name_for_node(&old_identifier),
        );
        let import_decl = self.factory().new_import_declaration(
            None,
            Some(self.factory().new_import_clause(
                SyntaxKind::Unknown,
                None,
                Some(self.factory().new_namespace_import(synth_name.clone())),
            )),
            updated_module_specifier.unwrap(),
            export_decl
                .attributes
                .clone()
                .map(|n| self.visitor().visit_node(n))
                .flatten(),
        );
        self.emit_context()
            .set_original(&import_decl, &export_decl.export_clause.clone().unwrap());

        let export_decl_node = if is_export_namespace_as_default_declaration(&node) {
            self.factory()
                .new_export_assignment(None, false, None, &synth_name)
        } else {
            let named_exports = self.factory().new_named_exports(&self.factory().new_node_list(vec![
                self.factory()
                    .new_export_specifier(false, Some(&synth_name), &old_identifier),
            ]));
            self.factory().new_export_declaration(
                None,
                false,
                &named_exports,
                None,
                None,
            )
        };
        self.emit_context().set_original(&export_decl_node, &node);
        single_or_many(
            Some(&[import_decl, export_decl_node]),
            &self.factory(),
        )
    }

    pub fn visit_call_expression(&mut self, node: Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("visit_call_expression"); 
        if self.compiler_options.rewrite_relative_import_extensions.is_true() {
            if (is_import_call(&node)
                && !node.as_call_expression().arguments.nodes.is_empty())
                || (is_in_js_file(&node)
                    && is_require_call(&node, false))
            {
                return self.visit_import_or_require_call(node);
            }
        }
        self.visitor().visit_each_child(node)
    }

    pub fn visit_import_or_require_call(&mut self, node: Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("visit_import_or_require_call"); 
        let call_expr = node.as_call_expression();
        if call_expr.arguments.nodes.is_empty() {
            return self.visitor().visit_each_child(node);
        }

        let expression = self.visitor().visit_node(call_expr.expression.clone());

        let argument = if is_string_literal_like(&call_expr.arguments.nodes[0]) {
            rewrite_module_specifier(
                self.emit_context(),
                Some(&call_expr.arguments.nodes[0]),
                self.compiler_options,
            )
        } else {
            Some(self.factory().new_rewrite_relative_import_extensions_helper(
                call_expr.arguments.nodes[0].clone(),
                self.compiler_options.jsx == JsxEmit::Preserve,
            ))
        };

        let mut arguments: Vec<Arc<Node>> = Vec::new();
        arguments.push(argument?);

        let rest = self
            .visitor()
            .visit_slice(&call_expr.arguments.nodes[1..]);
        arguments.extend(rest);

        let argument_list = Arc::new(NodeList {
            loc: call_expr.arguments.loc,
            nodes: arguments,
        });
        Some(self.factory().update_call_expression(
            &node,
            expression,
            call_expr.question_dot_token.clone(),
            None,
            argument_list,
            node.flags,
        ))
    }

    pub fn create_require_call(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("create_require_call"); 
        let module_name = get_external_module_name_literal(
            &self.factory(),
            node,
            self.current_source_file.as_ref(),
            None,
            None,
            self.compiler_options,
        );

        let mut args: Vec<Arc<Node>> = Vec::new();
        if let Some(module_name) = module_name {
            args.push(rewrite_module_specifier(
                self.emit_context(),
                Some(&module_name),
                self.compiler_options,
            )?);
        }

        if self.compiler_options.get_emit_module_kind() == ModuleKind::Preserve {
            let require_ident = self.factory().new_identifier("require");
            return Some(self.factory().new_call_expression(
                &require_ident,
                None,
                None,
                self.factory().new_node_list(args),
                NodeFlags::empty(),
            ));
        }

        if self.import_require_statements.is_none() {
            let create_require_name = self.factory().generated_name_node(
                &self.factory().new_unique_name_ex(
                    "_createRequire",
                    AutoGenerateOptions {
                        flags: GeneratedIdentifierFlags::OPTIMISTIC
                            | GeneratedIdentifierFlags::FILE_LEVEL,
                        ..Default::default()
                    },
                ),
            );
            let import_statement = self.factory().new_import_declaration(
                None,
                Some(self.factory().new_import_clause(
                    SyntaxKind::Unknown,
                    None,
                    Some(self.factory().new_named_imports(self.factory().new_node_list(vec![
                        self.factory().new_import_specifier(
                            false,
                            Some(self.factory().new_identifier("createRequire")),
                            create_require_name.clone(),
                        ),
                    ]))),
                )),
                self.factory()
                    .new_string_literal("module", TOKEN_FLAGS_NONE),
                None,
            );
            self.emit_context()
                .add_emit_flags(&import_statement, EmitFlags::CUSTOM_PROLOGUE);

            let require_helper_name = self.factory().generated_name_node(
                &self.factory().new_unique_name_ex(
                    "__require",
                    AutoGenerateOptions {
                        flags: GeneratedIdentifierFlags::OPTIMISTIC
                            | GeneratedIdentifierFlags::FILE_LEVEL,
                        ..Default::default()
                    },
                ),
            );
            let require_statement = self.factory().new_variable_statement(
                None,
                &self.factory().new_variable_declaration_list(
                    &self.factory().new_node_list(vec![self
                        .factory()
                        .new_variable_declaration(
                            &require_helper_name,
                            None,
                            None,
                            Some(&self.factory().new_call_expression(
                                &create_require_name,
                                None,
                                None,
                                self.factory().new_node_list(vec![self
                                    .factory()
                                    .new_property_access_expression(
                                        &self.factory().new_meta_property(
                                            SyntaxKind::ImportKeyword,
                                            self.factory().new_identifier("meta"),
                                        ),
                                        None,
                                        &self.factory().new_identifier("url"),
                                        NodeFlags::empty(),
                                    )]),
                                NodeFlags::empty(),
                            ))),
                        ]),
                    NodeFlags::Const,
                ),
            );
            self.emit_context()
                .add_emit_flags(&require_statement, EmitFlags::CUSTOM_PROLOGUE);
            self.import_require_statements = Some(ImportRequireStatements {
                statements: vec![import_statement, require_statement],
                require_helper_name,
            });
        }

        let require_helper_name = self
            .import_require_statements
            .as_ref()
            .unwrap()
            .require_helper_name
            .clone();
        Some(self.factory().new_call_expression(
            &require_helper_name,
            None,
            None,
            self.factory().new_node_list(args),
            NodeFlags::empty(),
        ))
    }
}

pub struct ImpliedModuleTransformer<'a> {
    pub opts: &'a TransformOptions<'a>,
    pub resolver: Arc<dyn ReferenceResolver>,
    pub get_emit_module_format_of_file: Arc<dyn Fn(&dyn HasFileName) -> ModuleKind + Send + Sync>,
    pub cjs_transformer: Option<Transformer>,
    pub esm_transformer: Option<Transformer>,
}

pub fn new_implied_module_transformer(opts: &TransformOptions) -> Transformer { ::tsox_core::fntrace::enter("new_implied_module_transformer"); 
    let _tx = ImpliedModuleTransformer {
        opts,
        resolver: opts.resolver.clone(),
        get_emit_module_format_of_file: opts.get_emit_module_format_of_file.clone(),
        cjs_transformer: None,
        esm_transformer: None,
    };
    Transformer::new(
        implied_module_transformer_visit,
        Some(opts.context.clone()),
    )
}

fn implied_module_transformer_visit(
    _transformer: &mut Transformer,
    node: Arc<Node>,
) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("implied_module_transformer_visit"); 
    Some(node)
}

impl<'a> ImpliedModuleTransformer<'a> {
    fn visit(&mut self, node: Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("visit"); 
        match node.kind {
            SyntaxKind::SourceFile => self.visit_source_file(node),
            _ => Some(node),
        }
    }

    pub fn visit_source_file(&mut self, node: Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("visit_source_file"); 
        if node.is_declaration_file() {
            return Some(node);
        }

        let format = (self.get_emit_module_format_of_file)(&empty_has_file_name());

        let transformer = if format >= ModuleKind::ES2015 {
            if self.esm_transformer.is_none() {
                self.esm_transformer = Some(new_es_module_transformer(self.opts));
            }
            self.esm_transformer.as_mut().unwrap()
        } else {
            unimplemented!(
                "CommonJsModuleTransformer(4 参数契约)与 Transformer 的桥接待后续接线"
            );
        };

        transformer.transform_source_file(node)
    }
}

pub fn is_declaration_name_of_enum_or_namespace(
    emit_context: EmitContext,
    node: &Arc<Node>,
) -> bool { ::tsox_core::fntrace::enter("is_declaration_name_of_enum_or_namespace"); 
    let original = emit_context.most_original(node);
    if let Some(parent) = original.parent() {
        if matches!(
            parent.kind,
            SyntaxKind::EnumDeclaration | SyntaxKind::ModuleDeclaration
        ) {
            return parent.name().is_some_and(|name| Arc::ptr_eq(name, &original));
        }
    }
    false
}

pub fn rewrite_module_specifier(
    mut emit_context: EmitContext,
    node: Option<&Arc<Node>>,
    compiler_options: &CompilerOptions,
) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("rewrite_module_specifier"); 
    let node = node?;
    if !is_string_literal(node) || !should_rewrite_module_specifier(node.text(), compiler_options)
    {
        return Some(node.clone());
    }
    let updated_text = change_extension(
        node.text(),
        &get_output_extension(node.text(), compiler_options.jsx),
    );
    if updated_text != node.text() {
        let updated = emit_context
            .factory()
            .new_string_literal(&updated_text, node.as_string_literal().token_flags);
        emit_context.set_original(&updated, node);
        emit_context.assign_comment_and_source_map_ranges(&updated, node);
        return Some(updated);
    }
    Some(node.clone())
}

pub fn create_empty_imports(factory: &NodeFactory) -> Arc<Node> { ::tsox_core::fntrace::enter("create_empty_imports"); 
    factory.new_export_declaration(
        None,
        false,
        &factory.new_named_exports(&factory.new_node_list(Vec::new())),
        None,
        None,
    )
}

pub fn get_external_module_name_literal(
    factory: &NodeFactory,
    import_node: &Arc<Node>,
    source_file: Option<&Arc<Node>>,
    host: Option<&EmitHost>,
    resolver: Option<EmitResolver>,
    compiler_options: &CompilerOptions,
) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("get_external_module_name_literal"); 
        let module_name = get_external_module_name(import_node)?;
        let mut name = try_get_module_name_from_declaration(
            import_node,
            host,
            factory,
            resolver,
            compiler_options,
        );
        if name.is_none() {
        let module_name_node = factory.new_string_literal(&module_name, TOKEN_FLAGS_NONE);
        name = try_rename_external_module(factory, &module_name_node, source_file?);
    }
    if name.is_none() {
        name = Some(factory.new_string_literal(&module_name, TOKEN_FLAGS_NONE));
    }
    name
}

pub fn try_get_module_name_from_file(
    factory: &NodeFactory,
    file: Option<&Arc<Node>>,
    host: Option<&EmitHost>,
    options: &CompilerOptions,
) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("try_get_module_name_from_file"); 
    let _file = file?;
    None
}

pub fn try_get_module_name_from_declaration(
    declaration: &Arc<Node>,
    host: Option<&EmitHost>,
    factory: &NodeFactory,
    resolver: Option<EmitResolver>,
    compiler_options: &CompilerOptions,
) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("try_get_module_name_from_declaration"); 
    let resolver = resolver?;
    let _ = resolver;
    None
}

pub fn get_external_module_name_from_path(
    host: &ResolveModuleNameResolutionHost,
    file_name: &str,
    reference_path: &str,
) -> String { ::tsox_core::fntrace::enter("get_external_module_name_from_path"); 
    String::new()
}

pub fn try_rename_external_module(
    factory: &NodeFactory,
    module_name: &Arc<Node>,
    source_file: &Arc<Node>,
) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("try_rename_external_module"); 
    None
}

pub fn is_file_level_reserved_generated_identifier(
    emit_context: EmitContext,
    name: &Arc<Node>,
) -> bool { ::tsox_core::fntrace::enter("is_file_level_reserved_generated_identifier"); 
    match emit_context.get_auto_generate_info(name) {
        Some(info) => {
            info.flags.is_file_level() && info.flags.is_optimistic() && info.flags.is_reserved_in_nested_scopes()
        }
        None => false,
    }
}

pub fn is_simple_inlineable_expression(expression: Option<&Arc<Node>>) -> bool { ::tsox_core::fntrace::enter("is_simple_inlineable_expression"); 
    match expression {
        Some(expression) => {
            !is_identifier(expression) && is_simple_copiable_expression(expression)
        }
        None => false,
    }
}

pub struct Transformer {
    emit_context: Option<EmitContext>,
    visitor: Option<NodeVisitor>,
}

impl Transformer {
    pub fn new(
        visit: fn(&mut Self, Arc<Node>) -> Option<Arc<Node>>,
        emit_context: Option<EmitContext>,
    ) -> Transformer { ::tsox_core::fntrace::enter("new"); 
        let emit_context = emit_context.unwrap_or_else(|| *EmitContext::new_emit_context());
        let visitor = emit_context.new_node_visitor(visit);
        Transformer {
            emit_context: Some(emit_context),
            visitor: Some(visitor),
        }
    }

    pub fn emit_context(&self) -> EmitContext { ::tsox_core::fntrace::enter("emit_context"); 
        self.emit_context.clone().unwrap()
    }

    pub fn visitor(&mut self) -> &mut NodeVisitor { ::tsox_core::fntrace::enter("visitor"); 
        self.visitor.as_mut().unwrap()
    }

    pub fn factory(&self) -> NodeFactory<'_> { ::tsox_core::fntrace::enter("factory"); 
        NodeFactory::new(self.emit_context.as_ref().unwrap())
    }

    pub fn transform_source_file(&mut self, file: Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("transform_source_file"); 
        Some(self.visitor().visit_node(&file))
    }
}
