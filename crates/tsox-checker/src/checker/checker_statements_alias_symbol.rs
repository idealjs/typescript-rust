#![allow(unused_imports)]

use crate::checker::checker_statements::*;

impl Checker {
    /// Go checkImportDeclaration/checkExportDeclaration 对每个别名绑定调用
    /// checkAliasSymbol；此处按声明形态下钻到对应绑定节点
    pub fn check_alias_symbol_bindings(&mut self, node: &Arc<Node>) {
        match &node.data {
            NodeData::ImportDeclaration(d) => {
                let Some(clause) = &d.import_clause else {
                    return;
                };
                let NodeData::ImportClause(ic) = &clause.data else {
                    return;
                };
                if ic.name.is_some() {
                    self.check_alias_symbol(clause);
                }
                if let Some(nb) = &ic.named_bindings {
                    match &nb.data {
                        NodeData::NamespaceImport(_) => self.check_alias_symbol(nb),
                        NodeData::NamedImports(ni) => {
                            for el in ni.elements.iter() {
                                self.check_alias_symbol(el);
                            }
                        }
                        _ => {}
                    }
                }
            }
            NodeData::ExportDeclaration(d) => {
                if let Some(clause) = &d.export_clause
                    && clause.kind == SyntaxKind::NamespaceExport
                {
                    self.check_alias_symbol(clause);
                }
            }
            NodeData::ImportEqualsDeclaration(_)
            | NodeData::NamespaceImport(_)
            | NodeData::ImportSpecifier(_)
            | NodeData::ExportSpecifier(_) => self.check_alias_symbol(node),
            _ => {}
        }
    }

    pub fn check_alias_symbol(&mut self, node: &Arc<Node>) {
        let Some(symbol) = self.program.symbol_map().symbol_of(node).cloned() else {
            return;
        };
        let Some(target) = self.resolve_alias_by_declaration(&symbol) else {
            return;
        };
        let symbol = symbol
            .export_symbol
            .clone()
            .unwrap_or_else(|| Arc::clone(&symbol));
        let target_flags = target.flags;

        let in_js_file = self
            .get_source_file_of_node(node)
            .is_some_and(|f| matches!(
                f.script_kind,
                tsox_frontend::ast::ScriptKind::Js | tsox_frontend::ast::ScriptKind::Jsx
            ));
        if in_js_file
            && !target_flags.intersects(SymbolFlags::VALUE)
            && !crate::checker::checker_get_excluded_symbol_flags::is_type_only_node(node)
        {
            self.check_js_type_only_alias(node, &symbol, &target);
            return;
        }

        let mut excluded_meanings = SymbolFlags::None;
        if symbol
            .flags
            .intersects(SymbolFlags::VALUE | SymbolFlags::ExportValue)
        {
            excluded_meanings |= SymbolFlags::VALUE;
        }
        if symbol.flags.intersects(SymbolFlags::TYPE) {
            excluded_meanings |= SymbolFlags::TYPE;
        }
        if symbol.flags.intersects(SymbolFlags::NAMESPACE) {
            excluded_meanings |= SymbolFlags::NAMESPACE;
        }

        if target_flags.intersects(excluded_meanings) {
            let message = if node.kind == SyntaxKind::ExportSpecifier {
                EXPORT_DECLARATION_CONFLICTS_WITH_EXPORTED_DECLARATION_OF_0
            } else {
                IMPORT_DECLARATION_CONFLICTS_WITH_LOCAL_DECLARATION_OF_0
            };
            // Go：锚定 import 的名字节点（默认导入名/命名空间导入名/导入别名）
            let anchor_loc = Self::import_conflict_anchor(node);
            self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                self.current_file.clone(),
                anchor_loc,
                message,
                vec![symbol.name.clone()],
            ));
        } else if node.kind != SyntaxKind::ExportSpecifier
            && self.compiler_options.isolated_modules
                == tsox_core::core::tristate::Tristate::True
            && symbol
                .flags
                .intersects(SymbolFlags::VALUE | SymbolFlags::ExportValue)
        {
            self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                self.current_file.clone(),
                node.loc,
                IMPORT_0_CONFLICTS_WITH_LOCAL_VALUE_SO_MUST_BE_DECLARED_WITH_A_TYPE_ONLY_IMPORT_WHEN_ISOLATEDMODULES_IS_ENABLED,
                vec![symbol.name.clone()],
            ));
        }
    }

    fn check_js_type_only_alias(
        &mut self,
        node: &Arc<Node>,
        symbol: &Arc<Symbol>,
        target: &Arc<Symbol>,
    ) {
        let error_node = property_name_or_name(node).unwrap_or_else(|| Arc::clone(node));
        if node.kind == SyntaxKind::ExportSpecifier {
            let mut diag = tsox_frontend::ast::Diagnostic::new(
                self.current_file.clone(),
                error_node.loc,
                TYPES_CANNOT_APPEAR_IN_EXPORT_DECLARATIONS_IN_JAVASCRIPT_FILES,
                vec![],
            );
            if let Some(file_sym) = self
                .get_source_file_of_node(node)
                .and_then(|f| self.program.symbol_map().symbol_of(&f.node).map(Arc::clone))
                && let Some(exported_name) = property_name_or_name(node)
                    .map(|n| n.text().to_string())
                && let Some(already) = file_sym.exports.entries.get(&exported_name)
                    && Arc::ptr_eq(already, target)
                    && let Some(decl) = already
                        .declarations
                        .iter()
                        .find(|d| d.kind == SyntaxKind::TypeAliasDeclaration)
            {
                diag.related_information
                    .push(tsox_frontend::ast::Diagnostic::new(
                        self.current_file.clone(),
                        decl.loc,
                        X_0_IS_AUTOMATICALLY_EXPORTED_HERE,
                        vec![already.name.clone()],
                    ));
            }
            self.diagnostics.add(diag);
            return;
        }

        let mut identifier_text = symbol.name.clone();
        if error_node.kind == SyntaxKind::Identifier {
            identifier_text = error_node.text().to_string();
        }
        let mut specifier_text = "...".to_string();
        if let Some(spec) = import_or_require_specifier_of(node) {
            specifier_text = spec;
        }
        let mut import_text = format!("import(\"{specifier_text}\")");
        if node.kind == SyntaxKind::ImportSpecifier {
            import_text = format!("{import_text}.{identifier_text}");
        }
        self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
            self.current_file.clone(),
            error_node.loc,
            X_0_IS_A_TYPE_AND_CANNOT_BE_IMPORTED_IN_JAVASCRIPT_FILES_USE_1_IN_A_JSDOC_TYPE_ANNOTATION,
            vec![identifier_text, import_text],
        ));
    }

    fn import_conflict_anchor(node: &Arc<Node>) -> tsox_core::core::text::TextRange {
        match &node.data {
            NodeData::ImportDeclaration(d) => {
                let mut name: Option<Arc<Node>> = None;
                if let Some(clause) = &d.import_clause
                    && let NodeData::ImportClause(ic) = &clause.data
                {
                    if let Some(n) = &ic.name {
                        name = Some(Arc::clone(n));
                    } else if let Some(nb) = &ic.named_bindings {
                        match &nb.data {
                            NodeData::NamespaceImport(ns) => name = Some(Arc::clone(&ns.name)),
                            NodeData::NamedImports(ni) => {
                                if let Some(first) = ni.elements.iter().next() {
                                    name = Some(Arc::clone(first));
                                }
                            }
                            _ => {}
                        }
                    }
                }
                name.map(|n| n.loc).unwrap_or(node.loc)
            }
            NodeData::ImportEqualsDeclaration(d) => d.name.loc,
            NodeData::NamespaceImport(d) => d.name.loc,
            NodeData::ImportClause(d) => {
                if let Some(n) = &d.name {
                    n.loc
                } else if let Some(nb) = &d.named_bindings {
                    Self::import_conflict_anchor(nb)
                } else {
                    node.loc
                }
            }
            NodeData::ImportSpecifier(_) => node.loc,
            _ => node.loc,
        }
    }
}

fn property_name_or_name(node: &Arc<Node>) -> Option<Arc<Node>> {
    match &node.data {
        NodeData::ImportSpecifier(d) => Some(
            d.property_name
                .clone()
                .unwrap_or_else(|| Arc::clone(&d.name)),
        ),
        NodeData::ExportSpecifier(d) => Some(
            d.property_name
                .clone()
                .unwrap_or_else(|| Arc::clone(&d.name)),
        ),
        NodeData::NamespaceImport(d) => Some(Arc::clone(&d.name)),
        NodeData::ImportClause(d) => d.name.clone(),
        NodeData::ImportEqualsDeclaration(d) => Some(Arc::clone(&d.name)),
        NodeData::NamespaceExport(d) => Some(Arc::clone(&d.name)),
        _ => None,
    }
}

/// Go TryGetModuleSpecifierFromDeclaration：沿祖先找 import/export/require
/// 声明并取其模块说明符
fn import_or_require_specifier_of(node: &Arc<Node>) -> Option<String> {
    let mut cur = Arc::clone(node);
    loop {
        let parent = cur.parent()?;
        match &parent.data {
            NodeData::ImportDeclaration(d) => {
                return module_specifier_text(&d.module_specifier);
            }
            NodeData::ExportDeclaration(d) => {
                return d.module_specifier.as_ref().and_then(module_specifier_text);
            }
            NodeData::ImportEqualsDeclaration(d) => {
                if let NodeData::ExternalModuleReference(ext) = &d.module_reference.data {
                    return module_specifier_text(&ext.expression);
                }
                return None;
            }
            NodeData::VariableDeclaration(d) => {
                if let Some(init) = &d.initializer
                    && let NodeData::CallExpression(call) = &init.data
                    && is_require_or_import_callee(&call.expression)
                    && let Some(first) = call.arguments.nodes.first()
                {
                    return module_specifier_text(first);
                }
                return None;
            }
            _ => {}
        }
        cur = parent;
    }
}

fn module_specifier_text(spec: &Arc<Node>) -> Option<String> {
    // Go isStringLiteralLike：无替换模板字面量同字符串（require(`./a`)）
    if !matches!(
        spec.kind,
        SyntaxKind::StringLiteral | SyntaxKind::NoSubstitutionTemplateLiteral
    ) {
        return None;
    }
    Some(spec.text().trim_matches(['"', '\'', '`']).to_string())
}

fn is_require_or_import_callee(callee: &Arc<Node>) -> bool {
    if callee.kind == SyntaxKind::ImportKeyword {
        return true;
    }
    matches!(&callee.data, NodeData::Identifier(i) if i.text == "require")
}
