#![allow(unused_imports)]

use super::*;

impl Binder {
    pub(crate) fn is_mutation_tracked_reference(&self, expr: &Arc<Node>) -> bool {
        match expr.kind {
            SyntaxKind::Identifier
            | SyntaxKind::ThisKeyword
            | SyntaxKind::SuperKeyword
            | SyntaxKind::MetaProperty => true,
            SyntaxKind::PropertyAccessExpression
            | SyntaxKind::ParenthesizedExpression
            | SyntaxKind::NonNullExpression => {
                if let Some(inner) = expr.expression() {
                    self.is_mutation_tracked_reference(&inner)
                } else {
                    false
                }
            }
            SyntaxKind::ElementAccessExpression => {
                if let NodeData::ElementAccessExpression(ea) = &expr.data {
                    if self.is_string_or_numeric_literal_like(&ea.argument_expression) {
                        return true;
                    }
                    return self.is_entity_name_expression(&ea.argument_expression)
                        && self.is_mutation_tracked_reference(&ea.expression);
                }
                false
            }
            _ => false,
        }
    }

    pub(crate) fn is_string_or_numeric_literal_like(&self, node: &Arc<Node>) -> bool {
        matches!(
            node.kind,
            SyntaxKind::StringLiteral
                | SyntaxKind::NumericLiteral
                | SyntaxKind::NoSubstitutionTemplateLiteral
        )
    }

    pub(crate) fn is_entity_name_expression(&self, node: &Arc<Node>) -> bool {
        matches!(
            node.kind,
            SyntaxKind::Identifier | SyntaxKind::QualifiedName
        )
    }

    pub(crate) fn bind_call_expression_flow(&mut self, node: &Arc<Node>) {
        if let NodeData::CallExpression(data) = &node.data {
            let expr = &data.expression;

            if let NodeData::PropertyAccessExpression(prop) = &expr.data {
                let name = self.node_text(&prop.name);
                if self.is_push_or_unshift_identifier(&name)
                    && self.is_mutation_tracked_reference(&prop.expression)
                {
                    let current = self.current_flow.clone();
                    if let Some(current) = current {
                        self.current_flow = Some(self.create_flow_mutation(&current, node));
                    }
                }
            }
        }
    }

    pub(crate) fn bind_this_property_assignment(&mut self, _node: &Arc<Node>) {}

    pub(crate) fn collect_expando_assignment(&mut self, node: &Arc<Node>) {
        let NodeData::BinaryExpression(bin) = &node.data else {
            return;
        };
        if bin.operator_token.kind != SyntaxKind::EqualsToken {
            return;
        }
        let base = match &bin.left.data {
            NodeData::PropertyAccessExpression(pae)
                if pae.expression.kind == SyntaxKind::Identifier
                    && pae.name.kind == SyntaxKind::Identifier =>
            {
                &pae.expression
            }
            NodeData::ElementAccessExpression(eae)
                if eae.expression.kind == SyntaxKind::Identifier =>
            {
                &eae.expression
            }
            _ => return,
        };

        let base_name = base.text();
        if matches!(base_name, "exports" | "module" | "globalThis") {
            return;
        }
        self.expando_assignments
            .push((Arc::clone(node), self.block_scope_container.clone()));
    }

    pub(crate) fn process_expando_assignments(&mut self) {
        let assignments = std::mem::take(&mut self.expando_assignments);
        for (node, scope_start) in assignments {
            let NodeData::BinaryExpression(bin) = &node.data else {
                continue;
            };
            let base = match &bin.left.data {
                NodeData::PropertyAccessExpression(pae) => &pae.expression,
                NodeData::ElementAccessExpression(eae) => &eae.expression,
                _ => continue,
            };
            let base_name = base.text();
            let mut target: Option<Arc<Symbol>> = None;
            let mut scope = scope_start;
            while let Some(sc) = scope {
                if let Some(sym) = self
                    .symbol_map
                    .locals
                    .get(&sc.id())
                    .and_then(|l| l.get(base_name))
                {
                    target = Some(Arc::clone(sym));
                    break;
                }

                if matches!(
                    sc.kind,
                    SyntaxKind::SourceFile | SyntaxKind::ModuleDeclaration
                ) && let Some(sym) = self.symbol_map.symbol_of(&sc)
                {
                    let hit = sym
                        .members
                        .get(base_name)
                        .or_else(|| sym.exports.get(base_name))
                        .cloned();
                    if let Some(h) = hit {
                        target = Some(h);
                        break;
                    }
                }
                scope = sc.parent.clone();
            }
            let Some(sym) = target else { continue };

            if !sym
                .value_declaration
                .as_ref()
                .is_some_and(|d| d.kind == SyntaxKind::FunctionDeclaration)
            {
                continue;
            }
            let member_name: Option<String> = match &bin.left.data {
                NodeData::PropertyAccessExpression(pae) => Some(pae.name.text().to_string()),
                NodeData::ElementAccessExpression(eae) => match &eae.argument_expression.data {
                    NodeData::StringLiteral(s) => Some(s.text.clone()),
                    NodeData::NumericLiteral(n) => Some(n.text.clone()),
                    _ => None,
                },
                _ => None,
            };
            match member_name {
                Some(mname) => {
                    let existing = sym
                        .exports
                        .get(&mname)
                        .or_else(|| sym.members.get(&mname))
                        .cloned()
                        .or_else(|| {
                            sym.declarations
                                .iter()
                                .filter(|d| d.kind == SyntaxKind::ModuleDeclaration)
                                .find_map(|md| {
                                    self.symbol_map
                                        .locals
                                        .get(&md.id())
                                        .and_then(|l| l.get(&mname))
                                        .cloned()
                                })
                        });
                    let eligible = existing.as_ref().map_or(true, |e| {
                        e.declarations
                            .iter()
                            .all(|d| d.kind == SyntaxKind::BinaryExpression)
                    });
                    if !eligible {
                        continue;
                    }
                    match existing {
                        Some(e) => {
                            let e_mut = Arc::as_ptr(&e) as *mut Symbol;
                            unsafe { (*e_mut).declarations.push(Arc::clone(&node)) };
                        }
                        None => {
                            let prop = self.new_symbol(SymbolFlags::Property, mname.clone());
                            let prop_mut = Arc::as_ptr(&prop) as *mut Symbol;
                            unsafe {
                                (*prop_mut).declarations.push(Arc::clone(&node));
                                (*prop_mut).parent = Some(Arc::clone(&sym));
                            }
                            let sym_mut = Arc::as_ptr(&sym) as *mut Symbol;
                            unsafe {
                                (*sym_mut).exports.insert(mname, prop);
                            }
                        }
                    }
                }
                None => {
                    let pseudo = sym
                        .exports
                        .get(crate::ast::INTERNAL_SYMBOL_NAME_ASSIGNMENT)
                        .cloned();
                    match pseudo {
                        Some(p) => {
                            let p_mut = Arc::as_ptr(&p) as *mut Symbol;
                            unsafe { (*p_mut).declarations.push(Arc::clone(&node)) };
                        }
                        None => {
                            let p = self.new_symbol(
                                SymbolFlags::empty(),
                                crate::ast::INTERNAL_SYMBOL_NAME_ASSIGNMENT.to_string(),
                            );
                            let p_mut = Arc::as_ptr(&p) as *mut Symbol;
                            unsafe {
                                (*p_mut).declarations.push(Arc::clone(&node));
                                (*p_mut).parent = Some(Arc::clone(&sym));
                            }
                            let sym_mut = Arc::as_ptr(&sym) as *mut Symbol;
                            unsafe {
                                (*sym_mut).exports.insert(
                                    crate::ast::INTERNAL_SYMBOL_NAME_ASSIGNMENT.to_string(),
                                    p,
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    pub(crate) fn bind_expression_statement(&mut self, node: &Arc<Node>) {
        if let NodeData::ExpressionStatement(data) = &node.data {
            self.bind(&data.expression);

            if let NodeData::BinaryExpression(bin_data) = &data.expression.data {
                if is_assignment_operator(bin_data.operator_token.kind) {
                    if let Some(current) = self.current_flow.take() {
                        let assign_flow = self.create_flow_assignment(&current, &data.expression);
                        self.symbol_map
                            .set_flow_node(&data.expression, Arc::clone(&assign_flow));
                        self.current_flow = Some(assign_flow);
                    }

                    if let NodeData::ElementAccessExpression(ea) = &bin_data.left.data {
                        if self.is_mutation_tracked_reference(&ea.expression) {
                            let current = self.current_flow.clone();
                            if let Some(current) = current {
                                self.current_flow =
                                    Some(self.create_flow_mutation(&current, &data.expression));
                            }
                        }
                    }
                }
            }

            if let NodeData::CallExpression(_) = &data.expression.data {
                if let Some(current) = self.current_flow.take() {
                    let call_flow = self.create_flow_call(&current, &data.expression);
                    self.symbol_map
                        .set_flow_node(&data.expression, Arc::clone(&call_flow));
                    self.current_flow = Some(call_flow);
                }
            }
        } else {
            self.bind_children(node);
        }
    }

    pub(crate) fn is_in_for_in_or_of_head(node: &Arc<Node>) -> bool {
        let Some(parent) = &node.parent else {
            return false;
        };
        let Some(grandparent) = &parent.parent else {
            return false;
        };
        matches!(
            grandparent.kind,
            SyntaxKind::ForInStatement | SyntaxKind::ForOfStatement
        )
    }
}
