#![allow(unused_imports)]

use crate::checker::checker_statements::*;

impl Checker {
    // Go checkExportAssignment isExportEquals 分支：
    // esm 实现文件与 esm 模式声明文件禁止 export=
    pub(crate) fn emit_implied_node_format_for_file(&self, file_name: &str) -> ModuleKind {
        use tsox_core::core::compiler_options::ModuleKind as M;
        let emit_kind = self.compiler_options.get_emit_module_kind();
        match emit_kind {
            M::Node16 | M::Node18 | M::Node20 | M::NodeNext => {
                tsox_tsoptions::tsoptions::implied_node_format_of_file(file_name, &|p| {
                    self.program.read_file(p)
                })
            }
            // Go GetImpliedNodeFormatForEmitWorker：非 Node 系 module 只有显式
            // ESM/CJS 扩展名或 package.json type 参与，plain .ts 回退 None
            _ => {
                let lower = file_name.to_ascii_lowercase();
                if lower.ends_with(".mts") || lower.ends_with(".mjs") {
                    M::ESNext
                } else if lower.ends_with(".cts") || lower.ends_with(".cjs") {
                    M::CommonJS
                } else {
                    M::None
                }
            }
        }
    }

    // Go getEmitSyntaxForUsageLocation：import 语句所在文件按 emit 格式归为
    // CJS/ESM/None，None 表示无法确定（合成 default 检查继续走声明文件规则）
    pub(crate) fn usage_emit_mode_for_import(&self, file_name: &str) -> ModuleKind {
        use tsox_core::core::compiler_options::ModuleKind as M;
        let emit = self.program.get_emit_module_format_of_file(file_name);
        if emit == M::CommonJS {
            M::CommonJS
        } else if emit.is_non_node_esm() || emit == M::Preserve {
            M::ESNext
        } else {
            M::None
        }
    }

    pub fn check_export_assignment_grammar(&mut self, node: &Arc<Node>) {
        let tsox_frontend::ast::NodeData::ExportAssignment(d) = &node.data else {
            return;
        };
        if let Some(modifiers) = &d.modifiers {
            if !modifiers.nodes.is_empty() && !self.check_grammar_modifiers(node) {
                self.grammar_error_on_node(
                    &modifiers.nodes[0],
                    &AN_EXPORT_ASSIGNMENT_CANNOT_HAVE_MODIFIERS,
                );
            }
        }
        if !d.is_export_equals {
            return;
        }
        // Go checkExportAssignment：非 ambient 的 export = 在
        // erasableSyntaxOnly 下报 TS1294（整节点 span）
        if !self.declaration_is_ambient(node) {
            self.erasable_syntax_error(node, node.loc);
        }
        let module_kind = self.compiler_options.get_emit_module_kind();
        let ambient = self.declaration_is_ambient(node);
        let implied_format = self
            .current_file
            .as_ref()
            .map(|f| self.emit_implied_node_format_for_file(&f.file_name))
            .unwrap_or(ModuleKind::None);
        if module_kind >= ModuleKind::ES2015
            && module_kind != ModuleKind::Preserve
            && ((ambient && implied_format == ModuleKind::ESNext)
                || (!ambient && implied_format != ModuleKind::CommonJS))
        {
            self.grammar_error_on_node(
                node,
                &EXPORT_ASSIGNMENT_CANNOT_BE_USED_WHEN_TARGETING_ECMASCRIPT_MODULES_CONSIDER_USING_EXPORT_DEFAULT_OR_ANOTHER_MODULE_FORMAT_INSTEAD,
            );
        } else if module_kind == ModuleKind::System && !ambient {
            self.grammar_error_on_node(
                node,
                &EXPORT_ASSIGNMENT_IS_NOT_SUPPORTED_WHEN_MODULE_FLAG_IS_SYSTEM,
            );
        }
    }

    // Go checkImportEqualsDeclaration：ESM 目标实现文件禁止 import = require
    pub fn check_import_equals_esm_grammar(&mut self, node: &Arc<Node>) {
        let tsox_frontend::ast::NodeData::ImportEqualsDeclaration(d) = &node.data else {
            return;
        };
        if d.module_reference.kind != SyntaxKind::ExternalModuleReference {
            return;
        }
        let module_kind = self.compiler_options.get_emit_module_kind();
        if !(ModuleKind::ES2015..=ModuleKind::ESNext).contains(&module_kind)
            || d.is_type_only
            || self.declaration_is_ambient(node)
        {
            return;
        }
        let Some(file) = self.current_file.clone() else {
            return;
        };
        let spec = match &d.module_reference.data {
            tsox_frontend::ast::NodeData::ExternalModuleReference(emr) => {
                emr.expression.text().to_string()
            }
            _ => return,
        };
        let resolved = self
            .program
            .resolve_external_module_path(&spec, &file.file_name, ModuleKind::None);
        if resolved.is_none() {
            return;
        }
        self.grammar_error_on_node(
            node,
            &IMPORT_ASSIGNMENT_CANNOT_BE_USED_WHEN_TARGETING_ECMASCRIPT_MODULES_CONSIDER_USING_IMPORT_ASTERISK_AS_NS_FROM_MOD_IMPORT_A_FROM_MOD_IMPORT_D_FROM_MOD_OR_ANOTHER_MODULE_FORMAT_INSTEAD,
        );
    }

    pub fn check_import_declaration_grammar(&mut self, node: &Arc<Node>) {
        let tsox_frontend::ast::NodeData::ImportDeclaration(d) = &node.data else {
            return;
        };
        let Some(modifiers) = &d.modifiers else {
            return;
        };
        if modifiers.nodes.is_empty() {
            return;
        }
        if !self.check_grammar_modifiers(node) {
            self.grammar_error_on_node(
                &modifiers.nodes[0],
                &AN_IMPORT_DECLARATION_CANNOT_HAVE_MODIFIERS,
            );
        }
    }

    pub fn check_import_ambient_rules(&mut self, node: &Arc<Node>) {
        if self.ambient_context_depth == 0 {
            let emit_format_cjs = self.current_file.as_ref().is_some_and(|f| {
                self.program.get_emit_module_format_of_file(&f.file_name)
                    < tsox_core::core::compiler_options::ModuleKind::System
            });
            let interop = self.compiler_options.es_module_interop.is_true_or_unknown();
            if emit_format_cjs {
                match &node.data {
                    tsox_frontend::ast::NodeData::ExportDeclaration(d)
                        if d.module_specifier.is_some() =>
                    {
                        match d.export_clause.as_ref().map(|c| c.kind) {
                            Some(SyntaxKind::NamespaceExport) if interop => {
                                self.check_external_emit_helpers(
                                    node,
                                    EXTERNAL_EMIT_HELPER_IMPORT_STAR,
                                );
                            }

                            None => {
                                self.check_external_emit_helpers(
                                    node,
                                    EXTERNAL_EMIT_HELPER_EXPORT_STAR,
                                );
                            }

                            Some(SyntaxKind::NamedImports | SyntaxKind::NamedExports) => {
                                let elements =
                                    d.export_clause.as_ref().and_then(|c| match &c.data {
                                        tsox_frontend::ast::NodeData::NamedExports(ne) => {
                                            Some(ne.elements.clone())
                                        }
                                        tsox_frontend::ast::NodeData::NamedImports(ni) => {
                                            Some(ni.elements.clone())
                                        }
                                        _ => None,
                                    });
                                if interop && let Some(elements) = elements {
                                    for spec in elements.nodes.iter() {
                                        if let tsox_frontend::ast::NodeData::ExportSpecifier(es) =
                                            &spec.data
                                        {
                                            let pn = es.property_name.as_ref().unwrap_or(&es.name);
                                            if pn.kind == SyntaxKind::DefaultKeyword
                                                || pn.text() == "default"
                                            {
                                                self.check_external_emit_helpers(
                                                    spec,
                                                    EXTERNAL_EMIT_HELPER_IMPORT_DEFAULT,
                                                );
                                            }
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    tsox_frontend::ast::NodeData::ImportDeclaration(d) => {
                        if let Some(clause) = &d.import_clause
                            && let tsox_frontend::ast::NodeData::ImportClause(ic) = &clause.data
                        {
                            if interop
                                && matches!(
                                    ic.named_bindings.as_ref().map(|b| b.kind),
                                    Some(SyntaxKind::NamespaceImport)
                                )
                            {
                                self.check_external_emit_helpers(
                                    node,
                                    EXTERNAL_EMIT_HELPER_IMPORT_STAR,
                                );
                            }

                            if interop
                                && let Some(nb) = &ic.named_bindings
                                && let tsox_frontend::ast::NodeData::NamedImports(ni) = &nb.data
                            {
                                for spec in ni.elements.nodes.iter() {
                                    if let tsox_frontend::ast::NodeData::ImportSpecifier(is) =
                                        &spec.data
                                    {
                                        let pn = is.property_name.as_ref().unwrap_or(&is.name);
                                        if pn.kind == SyntaxKind::DefaultKeyword
                                            || pn.text() == "default"
                                        {
                                            self.check_external_emit_helpers(
                                                spec,
                                                EXTERNAL_EMIT_HELPER_IMPORT_DEFAULT,
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}
