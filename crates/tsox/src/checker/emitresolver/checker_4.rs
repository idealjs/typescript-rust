#![allow(unused_imports)]

use super::*;

impl Checker {
    pub(crate) fn resolve_name_in_file_scope(&self, name: &str) -> Option<Arc<Symbol>> {
        let symbol_map = self.program.symbol_map();
        let file_id = self.current_file_id;
        if let Some(file_sym) = symbol_map.symbols.get(&file_id) {
            if let Some(sym) = file_sym.members.get(name) {
                return self.follow_alias(sym);
            }
        }

        for &container_id in self.scope_stack.iter().rev() {
            if let Some(locals) = symbol_map.locals.get(&container_id) {
                if let Some(sym) = locals.get(name) {
                    return self.follow_alias(sym);
                }
            }
            if let Some(container_sym) = symbol_map.symbols.get(&container_id) {
                if let Some(sym) = container_sym.members.get(name) {
                    return self.follow_alias(sym);
                }
            }
        }
        None
    }

    pub(crate) fn is_common_js_module_exports(node: &Arc<Node>) -> bool {
        if node.kind != SyntaxKind::BinaryExpression {
            return false;
        }

        let NodeData::BinaryExpression(bin) = &node.data else {
            return false;
        };
        let left_is_module_exports = matches!(&bin.left.data,
            NodeData::PropertyAccessExpression(pa)
                if pa.expression.kind == SyntaxKind::Identifier
                && pa.expression.text() == "module"
                && pa.name.text() == "exports");
        let left_is_exports_dot = matches!(&bin.left.data,
            NodeData::PropertyAccessExpression(pa)
                if pa.expression.kind == SyntaxKind::Identifier
                && pa.expression.text() == "exports");
        left_is_module_exports || left_is_exports_dot
    }

    pub(crate) fn export_specifier_name(node: &Arc<Node>) -> Option<Arc<Node>> {
        let NodeData::ExportSpecifier(d) = &node.data else {
            return None;
        };
        Some(if let Some(pn) = &d.property_name {
            Arc::clone(pn)
        } else {
            Arc::clone(&d.name)
        })
    }

    pub(crate) fn first_identifier_of(node: &Arc<Node>) -> Option<Arc<Node>> {
        let mut current = Arc::clone(node);
        loop {
            match &current.data {
                NodeData::Identifier(_) => return Some(current),
                NodeData::QualifiedName(q) => current = Arc::clone(&q.left),
                NodeData::PropertyAccessExpression(p) => current = Arc::clone(&p.expression),
                _ => return None,
            }
        }
    }

    pub fn is_entity_name_visible(
        &mut self,
        entity_name: &Arc<Node>,
        enclosing_declaration: &Arc<Node>,
    ) -> SymbolAccessibilityResult {
        let meaning = Self::meaning_of_entity_name_reference(entity_name);
        let first_identifier =
            Self::first_identifier_of(entity_name).unwrap_or_else(|| Arc::clone(entity_name));

        let symbol =
            self.resolve_name_in_enclosure(enclosing_declaration, first_identifier.text(), meaning);

        if let Some(sym) = &symbol {
            if sym.flags.contains(SymbolFlags::TypeParameter) && meaning.contains(SymbolFlags::TYPE)
            {
                return SymbolAccessibilityResult {
                    accessibility: SymbolAccessibility::Accessible,
                    ..Default::default()
                };
            }
        }

        let symbol = match symbol {
            Some(s) => s,
            None => {
                return SymbolAccessibilityResult {
                    accessibility: SymbolAccessibility::NotResolved,
                    error_symbol_name: first_identifier.text().to_string(),
                    error_node: Some(first_identifier),
                    ..Default::default()
                };
            }
        };

        match self.has_visible_declarations(&symbol) {
            Some(result) => result,
            None => SymbolAccessibilityResult {
                accessibility: SymbolAccessibility::NotAccessible,
                error_symbol_name: first_identifier.text().to_string(),
                error_node: Some(first_identifier),
                ..Default::default()
            },
        }
    }

    pub(crate) fn resolve_name_in_enclosure(
        &self,
        enclosing_declaration: &Arc<Node>,
        name: &str,
        meaning: SymbolFlags,
    ) -> Option<Arc<Symbol>> {
        let symbol_map = self.program.symbol_map();

        let mut current: Option<Arc<Node>> = Some(Arc::clone(enclosing_declaration));
        while let Some(n) = current {
            if let Some(sym) = symbol_map.symbol_of(&n) {
                if let Some(found) = sym.members.get(name) {
                    if found.flags.intersects(meaning) {
                        return self.follow_alias(found);
                    }
                }
            }
            if let Some(locals) = symbol_map.locals.get(&n.id()) {
                if let Some(found) = locals.get(name) {
                    if found.flags.intersects(meaning) {
                        return self.follow_alias(found);
                    }
                }
            }
            current = n.parent.clone();
        }

        if let Some(file_sym) = symbol_map.symbols.get(&self.current_file_id) {
            if let Some(found) = file_sym.members.get(name) {
                if found.flags.intersects(meaning) {
                    return self.follow_alias(found);
                }
            }
        }
        None
    }

    pub(crate) fn meaning_of_entity_name_reference(entity_name: &Arc<Node>) -> SymbolFlags {
        let parent = match &entity_name.parent {
            Some(p) => p,
            None => return SymbolFlags::TYPE,
        };

        let is_value_position = matches!(
            parent.kind,
            SyntaxKind::TypeQuery
                | SyntaxKind::ComputedPropertyName
                | SyntaxKind::BinaryExpression
                | SyntaxKind::ExpressionWithTypeArguments
        ) || (parent.kind == SyntaxKind::TypePredicate
            && matches!(&parent.data, NodeData::TypePredicateNode(tp) if tp.parameter_name.id() == entity_name.id()));
        if is_value_position {
            return SymbolFlags::VALUE | SymbolFlags::ExportValue;
        }

        let is_namespace_position = entity_name.kind == SyntaxKind::QualifiedName
            || entity_name.kind == SyntaxKind::PropertyAccessExpression
            || parent.kind == SyntaxKind::ImportEqualsDeclaration
            || (parent.kind == SyntaxKind::QualifiedName
                && matches!(&parent.data, NodeData::QualifiedName(q) if q.left.id() == entity_name.id()))
            || (parent.kind == SyntaxKind::PropertyAccessExpression
                && matches!(&parent.data, NodeData::PropertyAccessExpression(pa) if pa.expression.id() == entity_name.id()))
            || (parent.kind == SyntaxKind::ElementAccessExpression
                && matches!(&parent.data, NodeData::ElementAccessExpression(ea) if ea.expression.id() == entity_name.id()));
        if is_namespace_position {
            return SymbolFlags::NAMESPACE;
        }
        SymbolFlags::TYPE
    }

    pub fn has_visible_declarations(
        &mut self,
        symbol: &Arc<Symbol>,
    ) -> Option<SymbolAccessibilityResult> {
        let declarations = symbol.declarations.clone();
        for declaration in declarations.iter() {
            if declaration.kind == SyntaxKind::Identifier {
                continue;
            }
            if self.is_declaration_visible(declaration) {
                continue;
            }

            if let Some(any_import) = Checker::get_any_import_syntax(declaration) {
                let is_exported =
                    any_import.has_syntactic_modifier(crate::ast::ModifierFlags::Export);
                if !is_exported {
                    if let Some(parent) = any_import.parent.clone() {
                        if self.is_declaration_visible(&parent) {
                            self.declaration_links
                                .get_or_default(declaration)
                                .is_visible = true.into();
                            continue;
                        }
                    }
                }
            }

            if declaration.kind == SyntaxKind::VariableDeclaration {
                let var_list = declaration.parent.clone();
                let var_stmt = var_list.as_ref().and_then(|p| p.parent.clone());
                if let Some(vs) = &var_stmt {
                    if vs.kind == SyntaxKind::VariableStatement
                        && !vs.has_syntactic_modifier(crate::ast::ModifierFlags::Export)
                    {
                        if let Some(container) = vs.parent.clone() {
                            if self.is_declaration_visible(&container) {
                                self.declaration_links
                                    .get_or_default(declaration)
                                    .is_visible = true.into();
                                continue;
                            }
                        }
                    }
                }
            }

            if Checker::is_late_visibility_painted_statement(declaration)
                && !declaration.has_syntactic_modifier(crate::ast::ModifierFlags::Export)
            {
                if let Some(parent) = declaration.parent.clone() {
                    if self.is_declaration_visible(&parent) {
                        self.declaration_links
                            .get_or_default(declaration)
                            .is_visible = true.into();
                        continue;
                    }
                }
            }

            return None;
        }
        Some(SymbolAccessibilityResult {
            accessibility: SymbolAccessibility::Accessible,
            ..Default::default()
        })
    }

    pub fn get_enum_member_value_string(&mut self, node: &Arc<Node>) -> Option<String> {
        let NodeData::EnumMember(data) = &node.data else {
            return None;
        };
        let initializer = data.initializer.as_ref()?;
        match initializer.kind {
            SyntaxKind::StringLiteral => {
                if let NodeData::StringLiteral(s) = &initializer.data {
                    Some(format!("\"{}\"", s.text))
                } else {
                    None
                }
            }
            SyntaxKind::NumericLiteral => {
                if let NodeData::NumericLiteral(n) = &initializer.data {
                    Some(n.text.clone())
                } else {
                    None
                }
            }
            SyntaxKind::PrefixUnaryExpression => {
                if let NodeData::PrefixUnaryExpression(unary) = &initializer.data {
                    let operand_text = match &unary.operand.data {
                        NodeData::NumericLiteral(n) => n.text.clone(),
                        _ => return None,
                    };
                    let op = match unary.operator {
                        SyntaxKind::MinusToken => "-",
                        SyntaxKind::PlusToken => "+",
                        _ => return None,
                    };
                    Some(format!("{}{}", op, operand_text))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

}
