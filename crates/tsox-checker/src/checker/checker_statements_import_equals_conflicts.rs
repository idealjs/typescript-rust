#![allow(unused_imports)]

use crate::checker::checker_statements::*;

impl Checker {
    pub fn check_import_equals_conflicts(&mut self, node: &Arc<Node>) {
        self.check_external_import_in_namespace(node);
        if matches!(
            node.kind,
            SyntaxKind::ImportDeclaration | SyntaxKind::ImportEqualsDeclaration
        ) && self.ambient_context_depth > 0
            && self
                .current_file
                .as_ref()
                .is_some_and(|f| !f.file_name.starts_with("bundled://"))
        {
            let spec = match &node.data {
                tsox_frontend::ast::NodeData::ImportDeclaration(d) => {
                    Some(d.module_specifier.text().to_string())
                }
                tsox_frontend::ast::NodeData::ImportEqualsDeclaration(d) => {
                    if let tsox_frontend::ast::NodeData::ExternalModuleReference(ext) =
                        &d.module_reference.data
                    {
                        Some(ext.expression.text().to_string())
                    } else {
                        None
                    }
                }
                _ => None,
            };
            if let Some(spec) = spec {
                let relative = spec.starts_with("./")
                    || spec.starts_with("../")
                    || spec.starts_with(".\\")
                    || spec.starts_with("..\\");
                if relative {
                    let file = self.current_file.clone();
                    self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                            file,
                            node.loc,
                            tsox_core::diagnostics::messages_generated::
                                IMPORT_OR_EXPORT_DECLARATION_IN_AN_AMBIENT_MODULE_DECLARATION_CANNOT_REFERENCE_MODULE_THROUGH_RELATIVE_MODULE_NAME,
                            vec![],
                        ));

                    let spec_loc = match &node.data {
                        tsox_frontend::ast::NodeData::ImportDeclaration(d) => {
                            d.module_specifier.loc
                        }
                        tsox_frontend::ast::NodeData::ImportEqualsDeclaration(d) => {
                            if let tsox_frontend::ast::NodeData::ExternalModuleReference(ext) =
                                &d.module_reference.data
                            {
                                ext.expression.loc
                            } else {
                                d.module_reference.loc
                            }
                        }
                        _ => node.loc,
                    };
                    let spec_trimmed = spec.trim_matches(['"', '\'', '`']).to_string();
                    self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                            self.current_file.clone(),
                            spec_loc,
                            tsox_core::diagnostics::messages_generated::CANNOT_FIND_MODULE_0_OR_ITS_CORRESPONDING_TYPE_DECLARATIONS,
                            vec![spec_trimmed],
                        ));
                }
            }
        }

        if node.kind == SyntaxKind::ImportEqualsDeclaration
            && let tsox_frontend::ast::NodeData::ImportEqualsDeclaration(d) = &node.data
            && matches!(
                d.module_reference.kind,
                SyntaxKind::Identifier | SyntaxKind::QualifiedName
            )
        {
            let entity_ok = match &d.module_reference.data {
                tsox_frontend::ast::NodeData::Identifier(id) => {
                    is_valid_identifier_text(&id.text)
                        && !matches!(id.text.as_str(), "null" | "true" | "false")
                }
                _ => true,
            };

            let ns_hit = self
                .resolve_identifier_with_meaning(
                    &base_identifier_of(&d.module_reference),
                    SymbolFlags::NAMESPACE,
                )
                .map(|s| self.resolve_alias_base(s));
            let base_is_namespace = match &d.module_reference.data {
                tsox_frontend::ast::NodeData::Identifier(_) => ns_hit
                    .as_ref()
                    .is_some_and(|b| b.flags.intersects(SymbolFlags::NAMESPACE)),
                _ => true,
            };
            let traced_err = if entity_ok && !base_is_namespace {
                let base = base_identifier_of(&d.module_reference);
                let any_hit = self
                    .resolve_identifier(&base)
                    .map(|s| self.resolve_alias_base(s));
                let masked = any_hit.as_ref().is_some_and(|s| {
                    !s.flags.intersects(SymbolFlags::NAMESPACE)
                        && ns_hit
                            .as_ref()
                            .is_some_and(|n| n.flags.intersects(SymbolFlags::VALUE))
                });
                if masked {
                    ImportEntityError::HiddenByLocal(base)
                } else if any_hit
                    .as_ref()
                    .is_some_and(|s| s.flags.intersects(SymbolFlags::TYPE))
                {
                    ImportEntityError::TypeAsNamespace(base)
                } else {
                    ImportEntityError::NamespaceNotFound(base)
                }
            } else if entity_ok {
                match self.resolve_qualified_symbol_traced(&d.module_reference) {
                    Err((segment, ns_path, _member)) if ns_path.is_empty() => {
                        let any_hit = self
                            .resolve_identifier(&segment)
                            .map(|s| self.resolve_alias_base(s));
                        if any_hit
                            .as_ref()
                            .is_some_and(|s| s.flags.intersects(SymbolFlags::TYPE))
                        {
                            ImportEntityError::TypeAsNamespace(segment)
                        } else {
                            ImportEntityError::NamespaceNotFound(segment)
                        }
                    }
                    Err(e) => ImportEntityError::MissingMember(e),
                    Ok(_) => ImportEntityError::None,
                }
            } else {
                ImportEntityError::None
            };
            if !matches!(traced_err, ImportEntityError::None)
                && self
                    .current_file
                    .as_ref()
                    .is_some_and(|f| !f.file_name.starts_with("bundled://"))
            {
                let file = self.current_file.clone();
                match traced_err {
                    ImportEntityError::None => {}
                    ImportEntityError::NamespaceNotFound(seg) => {
                        self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                            file,
                            seg.loc,
                            tsox_core::diagnostics::messages_generated::CANNOT_FIND_NAMESPACE_0,
                            vec![seg.text().to_string()],
                        ));
                    }
                    ImportEntityError::TypeAsNamespace(seg) => {
                        self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                                file,
                                seg.loc,
                                tsox_core::diagnostics::messages_generated::
                                    X_0_ONLY_REFERS_TO_A_TYPE_BUT_IS_BEING_USED_AS_A_NAMESPACE_HERE,
                                vec![seg.text().to_string()],
                            ));
                    }
                    ImportEntityError::HiddenByLocal(seg) => {
                        self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                                file,
                                seg.loc,
                                tsox_core::diagnostics::messages_generated::
                                    MODULE_0_IS_HIDDEN_BY_A_LOCAL_DECLARATION_WITH_THE_SAME_NAME,
                                vec![seg.text().to_string()],
                            ));
                    }
                    ImportEntityError::MissingMember((seg, ns_path, member)) => {
                        self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                                file,
                                seg.loc,
                                tsox_core::diagnostics::messages_generated::
                                    NAMESPACE_0_HAS_NO_EXPORTED_MEMBER_1,
                                vec![ns_path, member],
                            ));
                    }
                }
            }

            if entity_ok
                && let Some(ns) = ns_hit.as_ref()
                && ns.flags.intersects(SymbolFlags::VALUE)
            {
                let base = base_identifier_of(&d.module_reference);
                let masked = self
                    .resolve_identifier_with_meaning(
                        &base,
                        SymbolFlags::VALUE | SymbolFlags::NAMESPACE,
                    )
                    .map(|s| self.resolve_alias_base(s))
                    .is_some_and(|s| !s.flags.intersects(SymbolFlags::NAMESPACE));
                if masked {
                    self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                            self.current_file.clone(),
                            base.loc,
                            tsox_core::diagnostics::messages_generated::
                                MODULE_0_IS_HIDDEN_BY_A_LOCAL_DECLARATION_WITH_THE_SAME_NAME,
                            vec![base.text().to_string()],
                        ));
                }
            }

            if node.kind == SyntaxKind::ImportEqualsDeclaration
                && let tsox_frontend::ast::NodeData::ImportEqualsDeclaration(d) = &node.data
            {
                if let Some(alias_sym) = self.program.symbol_map().symbol_of(node).cloned() {
                    let target = self.resolve_alias_base(Arc::clone(&alias_sym));

                    let target_resolved = !Arc::ptr_eq(&target, &alias_sym)
                        || !target.flags.intersects(SymbolFlags::Alias);
                    if target_resolved && target.flags.intersects(SymbolFlags::TYPE) {
                        self.check_reserved_type_name(
                            &d.name,
                            &tsox_core::diagnostics::messages_generated::IMPORT_NAME_CANNOT_BE_0,
                        );
                    }
                }
            }
        }
    }
}

impl Checker {
    /// Go checkExternalImportOrExportDeclaration：import/export 声明带外部
    /// 模块名且不在文件顶层、也不在 ambient 模块块内时，import 形态报
    /// TS1147；嵌套（namespace 内）的 import= 声明不在 program 级扫描
    /// （collectModuleReferences 仅遍历顶层），此处补报模块不可解析 TS2307
    fn check_external_import_in_namespace(&mut self, node: &Arc<Node>) {
        if !matches!(
            node.kind,
            SyntaxKind::ImportDeclaration | SyntaxKind::ImportEqualsDeclaration | SyntaxKind::ExportDeclaration
        ) {
            return;
        }
        let module_name = match &node.data {
            tsox_frontend::ast::NodeData::ImportDeclaration(d) => Some(d.module_specifier.clone()),
            tsox_frontend::ast::NodeData::ExportDeclaration(d) => d.module_specifier.clone(),
            tsox_frontend::ast::NodeData::ImportEqualsDeclaration(d) => {
                if let tsox_frontend::ast::NodeData::ExternalModuleReference(ext) =
                    &d.module_reference.data
                {
                    Some(ext.expression.clone())
                } else {
                    None
                }
            }
            _ => None,
        };
        let Some(module_name) = module_name else {
            return;
        };
        if module_name.kind != SyntaxKind::StringLiteral {
            return;
        }
        let parent = node.parent();
        let in_ambient_external_module = parent.as_ref().is_some_and(|p| {
            p.kind == SyntaxKind::ModuleBlock
                && p
                    .parent()
                    .is_some_and(|gp| tsox_frontend::ast::is_ambient_module(&gp))
        });
        if parent.is_none()
            || parent.is_some_and(|p| p.kind == SyntaxKind::SourceFile)
            || in_ambient_external_module
        {
            return;
        }
        let message = if node.kind == SyntaxKind::ExportDeclaration {
            tsox_core::diagnostics::messages_generated::
                EXPORT_DECLARATIONS_ARE_NOT_PERMITTED_IN_A_NAMESPACE
        } else {
            tsox_core::diagnostics::messages_generated::
                IMPORT_DECLARATIONS_IN_A_NAMESPACE_CANNOT_REFERENCE_A_MODULE
        };
        self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
            self.current_file.clone(),
            module_name.loc,
            message,
            Vec::new(),
        ));
        if node.kind == SyntaxKind::ImportEqualsDeclaration {
            let spec = module_name.text().trim_matches(['"', '\'', '`']).to_string();
            if self.resolve_module_file_symbol(&spec).is_none() {
                let (message, args) =
                    tsox_frontend::parser::cannot_resolve_module_error(&self.compiler_options, &spec);
                self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                    self.current_file.clone(),
                    module_name.loc,
                    message.clone(),
                    args,
                ));
            }
        }
    }
}
