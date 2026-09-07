#![allow(unused_imports)]

use crate::checker::checker_statements::*;

impl Checker {
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
