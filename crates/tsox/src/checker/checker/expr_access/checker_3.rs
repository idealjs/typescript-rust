#![allow(unused_imports)]

use super::*;

impl Checker {
    pub(crate) fn lookup_private_identifier_declaration(
        &self,
        text: &str,
        location: &Arc<Node>,
    ) -> Option<Arc<Symbol>> {
        let symbol_map = self.program.symbol_map();
        let mut current = Some(Arc::clone(location));
        while let Some(n) = current {
            if matches!(
                n.kind,
                SyntaxKind::ClassDeclaration | SyntaxKind::ClassExpression
            ) {
                if let Some(sym) = symbol_map.symbol_of(&n) {
                    if let Some(prop) = sym.members.get(text) {
                        return Some(Arc::clone(prop));
                    }
                    if let Some(prop) = sym.exports.get(text) {
                        return Some(Arc::clone(prop));
                    }
                }
            }
            current = n.parent.clone();
        }
        None
    }

    pub(crate) fn is_ancestor_class_of(&self, node: &Arc<Node>, ancestor: &Arc<Node>) -> bool {
        let mut current = Some(Arc::clone(node));
        while let Some(n) = current {
            if Arc::ptr_eq(&n, ancestor) {
                return true;
            }
            current = n.parent.clone();
        }
        false
    }

    pub(crate) fn check_private_identifier_access(
        &mut self,
        node: &Arc<Node>,
        name: &Arc<Node>,
        name_text: &str,
        obj_type: &Arc<Type>,
    ) -> bool {
        let assignment_kind = crate::checker::utilities::get_assignment_target_kind(node);
        let lexical = self.lookup_private_identifier_declaration(name_text, name);

        if assignment_kind != crate::checker::utilities::AssignmentKind::None
            && let Some(lx) = &lexical
            && lx
                .declarations
                .iter()
                .any(|d| d.kind == SyntaxKind::MethodDeclaration)
        {
            self.diagnostics.add(crate::ast::Diagnostic::new(
                self.current_file.clone(),
                name.loc,
                crate::diagnostics::messages_generated::
                    CANNOT_ASSIGN_TO_PRIVATE_METHOD_0_PRIVATE_METHODS_ARE_NOT_WRITABLE,
                vec![name_text.to_string()],
            ));
        }

        let type_member: Option<Arc<Symbol>> = obj_type
            .as_structured()
            .and_then(|s| s.members.get(name_text))
            .map(Arc::clone);
        let resolved = match (&lexical, &type_member) {
            (Some(lx), Some(m)) => {
                let same_decl = lx
                    .declarations
                    .iter()
                    .any(|ld| m.declarations.iter().any(|d| d.id() == ld.id()));
                let same_class = lx
                    .declarations
                    .first()
                    .and_then(|ld| ld.parent.clone())
                    .zip(m.declarations.first().and_then(|d| d.parent.clone()))
                    .is_some_and(|(a, b)| a.id() == b.id());
                let synthetic_same_class = m.declarations.is_empty()
                    && lx
                        .declarations
                        .first()
                        .and_then(|d| d.parent.clone())
                        .and_then(|class| self.program.symbol_map().symbol_of(&class))
                        .zip(obj_type.symbol.clone())
                        .is_some_and(|(a, b)| Arc::ptr_eq(&a, &b));
                (same_decl || same_class || synthetic_same_class).then(|| Arc::clone(m))
            }
            _ => None,
        };

        if resolved.is_none() {
            let property_on_type = type_member.as_ref().filter(|m| {
                m.declarations.iter().any(|d| {
                    d.name()
                        .is_some_and(|n| n.kind == SyntaxKind::PrivateIdentifier)
                })
            });
            if let Some(property) = property_on_type {
                let type_class = self.declaring_class_of_private_member(property);
                if let (Some(lx), Some(type_class)) = (&lexical, &type_class) {
                    let lexical_class = self.declaring_class_of_private_member(lx);
                    if lexical_class.is_some_and(|lc| self.is_ancestor_class_of(&lc, type_class)) {
                        let type_str = self.type_to_string(obj_type);
                        self.diagnostics.add(crate::ast::Diagnostic::new(
                            self.current_file.clone(),
                            name.loc,
                            crate::diagnostics::messages_generated::THE_PROPERTY_0_CANNOT_BE_ACCESSED_ON_TYPE_1_WITHIN_THIS_CLASS_BECAUSE_IT_IS_SHADOWED_BY_ANOTHER_PRIVATE_IDENTIFIER_WITH_THE_SAME_SPELLING,
                            vec![name_text.to_string(), type_str],
                        ));
                        return true;
                    }
                }
                let class_name = type_class.map_or_else(
                    || "(anonymous)".to_string(),
                    |c| match &c.data {
                        crate::ast::NodeData::ClassDeclaration(d) => d
                            .name
                            .as_ref()
                            .map(|n| n.text().to_string())
                            .unwrap_or_else(|| "(anonymous)".to_string()),
                        _ => "(anonymous)".to_string(),
                    },
                );
                self.diagnostics.add(crate::ast::Diagnostic::new(
                    self.current_file.clone(),
                    name.loc,
                    crate::diagnostics::messages_generated::PROPERTY_0_IS_NOT_ACCESSIBLE_OUTSIDE_CLASS_1_BECAUSE_IT_HAS_A_PRIVATE_IDENTIFIER,
                    vec![name_text.to_string(), class_name],
                ));
                return true;
            }
            return false;
        }

        let setonly = resolved.as_ref().is_some_and(|m| {
            m.flags.contains(SymbolFlags::SetAccessor)
                && !m.flags.contains(SymbolFlags::GetAccessor)
        });
        if setonly && assignment_kind != crate::checker::utilities::AssignmentKind::Definite {
            self.diagnostics.add(crate::ast::Diagnostic::new(
                self.current_file.clone(),
                node.loc,
                crate::diagnostics::messages_generated::PRIVATE_ACCESSOR_WAS_DEFINED_WITHOUT_A_GETTER,
                vec![],
            ));
        }
        false
    }

    pub(crate) fn is_within_declaring_class(&self, class_node: &Arc<Node>) -> bool {
        self.enclosing_class_stack
            .iter()
            .any(|c| Arc::ptr_eq(c, class_node))
    }

    pub(crate) fn super_in_computed_name_of_innermost_class(&self, node: &Arc<Node>) -> bool {
        let Some(innermost) = self.enclosing_class_stack.last() else {
            return false;
        };
        let mut in_computed_name = false;
        let mut cur = node.parent.as_ref();
        while let Some(c) = cur {
            if Arc::ptr_eq(c, innermost) {
                return in_computed_name;
            }
            if c.kind == SyntaxKind::ComputedPropertyName {
                in_computed_name = true;
            }
            if matches!(
                c.kind,
                SyntaxKind::ClassDeclaration | SyntaxKind::ClassExpression
            ) {
                return false;
            }
            cur = c.parent.as_ref();
        }
        false
    }

    pub(crate) fn function_body_definitely_returns(&self, body: &Arc<Node>) -> bool {
        if body.kind != SyntaxKind::Block {
            return false;
        }
        if let crate::ast::NodeData::Block(data) = &body.data {
            if let Some(last) = data.statements.nodes.last() {
                return self.statement_always_returns(last);
            }
        }
        false
    }

    pub(crate) fn statement_always_returns(&self, stmt: &Arc<Node>) -> bool {
        match stmt.kind {
            SyntaxKind::ReturnStatement | SyntaxKind::ThrowStatement => true,
            SyntaxKind::Block => {
                if let crate::ast::NodeData::Block(data) = &stmt.data {
                    if let Some(last) = data.statements.nodes.last() {
                        return self.statement_always_returns(last);
                    }
                }
                false
            }
            SyntaxKind::IfStatement => {
                if let crate::ast::NodeData::IfStatement(data) = &stmt.data {
                    let then_returns = self.statement_always_returns(&data.then_statement);
                    let else_returns = data
                        .else_statement
                        .as_ref()
                        .map_or(false, |e| self.statement_always_returns(e));
                    then_returns && else_returns
                } else {
                    false
                }
            }

            SyntaxKind::WhileStatement | SyntaxKind::DoStatement => {
                let (condition, body) = match &stmt.data {
                    crate::ast::NodeData::WhileStatement(data) => {
                        (&data.expression, &data.statement)
                    }
                    crate::ast::NodeData::DoStatement(data) => (&data.expression, &data.statement),
                    _ => return false,
                };
                condition.kind == SyntaxKind::TrueKeyword
                    && !Self::loop_has_escaping_break(body, true)
            }

            SyntaxKind::ForStatement => {
                if let crate::ast::NodeData::ForStatement(data) = &stmt.data {
                    data.condition
                        .as_ref()
                        .map_or(true, |c| c.kind == SyntaxKind::TrueKeyword)
                        && !Self::loop_has_escaping_break(&data.statement, true)
                } else {
                    false
                }
            }

            SyntaxKind::SwitchStatement => {
                if let crate::ast::NodeData::SwitchStatement(data) = &stmt.data
                    && let crate::ast::NodeData::CaseBlock(block) = &data.case_block.data
                {
                    let has_default = block
                        .clauses
                        .iter()
                        .any(|c| c.kind == SyntaxKind::DefaultClause);
                    if !has_default {
                        return false;
                    }
                    block.clauses.iter().all(|c| match &c.data {
                        crate::ast::NodeData::CaseOrDefaultClause(cd) => cd
                            .statements
                            .nodes
                            .last()
                            .map_or(true, |l| self.statement_always_returns(l)),
                        _ => false,
                    })
                } else {
                    false
                }
            }
            _ => false,
        }
    }
}
