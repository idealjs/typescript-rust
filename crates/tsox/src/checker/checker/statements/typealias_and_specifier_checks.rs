#![allow(unused_imports)]

use super::*;

impl Checker {
    pub fn check_type_alias_and_specifiers(&mut self, node: &Arc<Node>) {
            if matches!(
                node.kind,
                SyntaxKind::ImportDeclaration | SyntaxKind::ExportDeclaration
            ) && self.ambient_context_depth == 0
                && self
                    .current_file
                    .as_ref()
                    .is_none_or(|f| !f.file_name.starts_with("bundled://"))
            {
                self.check_module_specifier_members(node);
                self.check_module_export_names(node);
            }

            if matches!(
                node.kind,
                SyntaxKind::ImportDeclaration
                    | SyntaxKind::ExportDeclaration
                    | SyntaxKind::ImportEqualsDeclaration
            ) && self.ambient_context_depth == 0
                && self
                    .current_file
                    .as_ref()
                    .is_none_or(|f| !f.file_name.starts_with("bundled://"))
            {
                self.check_module_format_mismatch(node);
            }

            if node.kind == SyntaxKind::TypeAliasDeclaration
                && let crate::ast::NodeData::TypeAliasDeclaration(d) = &node.data
            {
                self.check_type_annotation(&d.type_node);

                if !self
                    .current_file
                    .as_ref()
                    .is_some_and(|f| f.file_name.starts_with("bundled://"))
                {
                    let _ = self.get_type_from_type_node(&d.type_node);
                }
            }

            {
                use crate::core::compiler_options::ModuleKind;
                let module_ok = matches!(
                    self.compiler_options.module,
                    ModuleKind::ESNext
                        | ModuleKind::Node18
                        | ModuleKind::Node20
                        | ModuleKind::NodeNext
                        | ModuleKind::Preserve
                );
                let attributes = match &node.data {
                    crate::ast::NodeData::ImportDeclaration(d) => d.attributes.clone(),
                    crate::ast::NodeData::ExportDeclaration(d) => d.attributes.clone(),
                    _ => None,
                };
                if let Some(attrs) = attributes {
                    let file_has_parse_errors = self
                        .current_file
                        .as_ref()
                        .is_some_and(|f| f.has_parse_diagnostics);
                    if !file_has_parse_errors {
                        let file = self.current_file.clone();
                        let is_type_only = match &node.data {
                            crate::ast::NodeData::ImportDeclaration(d) => {
                                d.import_clause.as_ref().is_some_and(|c| {
                                    matches!(
                                        &c.data,
                                        crate::ast::NodeData::ImportClause(ic)
                                            if ic.phase_modifier
                                                == Some(SyntaxKind::TypeKeyword)
                                    )
                                })
                            }
                            crate::ast::NodeData::ExportDeclaration(d) => d.is_type_only,
                            _ => false,
                        };
                        let override_mode =
                            self.get_resolution_mode_override(&attrs, is_type_only);
                        let exempt = is_type_only && override_mode.is_some();
                        if !exempt {
                            let emit_commonjs = file
                                .as_ref()
                                .map(|f| {
                                    self.program.get_emit_module_format_of_file(&f.file_name)
                                        == ModuleKind::CommonJS
                                })
                                .unwrap_or(false);
                            if !module_ok {
                                self.diagnostics.add(crate::ast::Diagnostic::new(
                                    file,
                                    attrs.loc,
                                    crate::diagnostics::messages_generated::
                                        IMPORT_ATTRIBUTES_ARE_ONLY_SUPPORTED_WHEN_THE_MODULE_OPTION_IS_SET_TO_ESNEXT_NODE18_NODE20_NODENEXT_OR_PRESERVE,
                                    Vec::new(),
                                ));
                            } else if emit_commonjs {
                                self.diagnostics.add(crate::ast::Diagnostic::new(
                                    file,
                                    attrs.loc,
                                    crate::diagnostics::messages_generated::
                                        IMPORT_ATTRIBUTES_ARE_NOT_ALLOWED_ON_STATEMENTS_THAT_COMPILE_TO_COMMONJS_REQUIRE_CALLS,
                                    Vec::new(),
                                ));
                            } else if is_type_only {
                                self.diagnostics.add(crate::ast::Diagnostic::new(
                                    file,
                                    attrs.loc,
                                    crate::diagnostics::messages_generated::
                                        IMPORT_ATTRIBUTES_CANNOT_BE_USED_WITH_TYPE_ONLY_IMPORTS_OR_EXPORTS,
                                    Vec::new(),
                                ));
                            } else if override_mode.is_some() {
                                self.diagnostics.add(crate::ast::Diagnostic::new(
                                    file,
                                    attrs.loc,
                                    crate::diagnostics::messages_generated::
                                        X_RESOLUTION_MODE_CAN_ONLY_BE_SET_FOR_TYPE_ONLY_IMPORTS,
                                    Vec::new(),
                                ));
                            }
                        }
                    }
                }
            }
    }
}