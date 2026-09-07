#![allow(unused_imports)]

use super::*;

impl Checker {
    pub(crate) fn check_assignment_compat(
        &mut self,
        node: &Arc<Node>,
        data: &crate::ast::node_data_generated::BinaryExpressionData,
    ) {
        use crate::ast::SyntaxKind::*;

        if data.operator_token.kind == EqualsToken
            && matches!(
                data.left.kind,
                ObjectLiteralExpression | ArrayLiteralExpression
            )
        {
            return;
        }

        let mut target: &Arc<Node> = &data.left;
        loop {
            match &target.data {
                crate::ast::NodeData::ParenthesizedExpression(p) => {
                    target = &p.expression;
                }
                crate::ast::NodeData::NonNullExpression(n) => {
                    target = &n.expression;
                }
                _ => break,
            }
        }

        let optional_chain = match &target.data {
            crate::ast::NodeData::PropertyAccessExpression(pa) => pa.question_dot_token.is_some(),
            crate::ast::NodeData::ElementAccessExpression(ea) => ea.question_dot_token.is_some(),
            _ => false,
        };
        let is_reference = matches!(
            target.kind,
            Identifier | PropertyAccessExpression | ElementAccessExpression
        );
        if !is_reference || optional_chain {
            let message = if optional_chain {
                crate::diagnostics::messages_generated::
                    THE_LEFT_HAND_SIDE_OF_AN_ASSIGNMENT_EXPRESSION_MAY_NOT_BE_AN_OPTIONAL_PROPERTY_ACCESS
            } else {
                crate::diagnostics::messages_generated::
                    THE_LEFT_HAND_SIDE_OF_AN_ASSIGNMENT_EXPRESSION_MUST_BE_A_VARIABLE_OR_A_PROPERTY_ACCESS
            };

            let loc = if data.left.kind == SyntaxKind::ParenthesizedExpression {
                node.loc
            } else {
                target.loc
            };
            self.diagnostics.add(crate::ast::Diagnostic::new(
                self.current_file.clone(),
                loc,
                message,
                Vec::new(),
            ));

            self.check_expression(&data.left);
            return;
        }
        let Some(left_type) = self.assignment_target_type(target) else {
            return;
        };

        if target.kind == Identifier {
            if let Some(sym) = self.resolve_identifier(target) {
                let base = self.resolve_alias_base(sym);
                if base
                    .flags
                    .intersects(SymbolFlags::Class | SymbolFlags::ENUM | SymbolFlags::ValueModule)
                    && !base.flags.intersects(
                        SymbolFlags::VARIABLE
                            | SymbolFlags::PROPERTY_OR_ACCESSOR
                            | SymbolFlags::Function,
                    )
                {
                    return;
                }

                if self.symbol_is_const_variable(&base) {
                    return;
                }
            }
        }

        if left_type.flags.contains(TypeFlags::Any) && left_type.intrinsic_name() == Some("error") {
            return;
        }

        if self.assignment_target_is_readonly(target) {
            return;
        }
        let right_type = match data.operator_token.kind {
            EqualsToken => self.get_type_of_node(&data.right),

            AmpersandAmpersandEqualsToken | BarBarEqualsToken | QuestionQuestionEqualsToken => {
                match self.logical_rhs_frame(data.operator_token.kind, target) {
                    Some((sym, t)) => {
                        self.logical_rhs_narrowing_frames.push((sym, t));
                        let rt = self.get_type_of_node(&data.right);
                        self.logical_rhs_narrowing_frames.pop();
                        rt
                    }
                    None => self.get_type_of_node(&data.right),
                }
            }

            _ => {
                if self
                    .arith_operand_error_nodes
                    .contains(&(Arc::as_ptr(node) as *const crate::ast::Node))
                {
                    return;
                }
                self.get_type_of_node(node)
            }
        };

        let _ = self.check_type_assignable_to_and_optionally_elaborate(
            &right_type,
            &left_type,
            Some(target),
            Some(&data.right),
            None,
            None,
        );
    }

    pub(crate) fn write_type_of_property_symbol(
        &mut self,
        prop: &Arc<crate::ast::Symbol>,
    ) -> Arc<Type> {
        if prop.flags.contains(SymbolFlags::SetAccessor)
            && let Some(setter) = prop
                .declarations
                .iter()
                .find(|d| d.kind == SyntaxKind::SetAccessor)
            && let crate::ast::NodeData::SetAccessorDeclaration(sd) = &setter.data
            && let Some(param) = sd.parameters.iter().next()
            && let crate::ast::NodeData::ParameterDeclaration(pd) = &param.data
            && let Some(tn) = &pd.type_node
        {
            return self.get_type_from_type_node(tn);
        }
        self.get_type_of_symbol(prop)
    }

    pub(crate) fn assignment_target_type(&mut self, target: &Arc<Node>) -> Option<Arc<Type>> {
        match &target.data {
            crate::ast::NodeData::Identifier(_) => {
                let sym = self.resolve_identifier(target)?;
                let declared = self.get_type_of_symbol(&sym);

                let target_kind = get_assignment_target_kind(target);
                let compound_like = target_kind == AssignmentKind::Definite
                    && is_in_compound_like_assignment(target);
                if compound_like || target_kind == AssignmentKind::Compound {
                    Some(self.get_base_type_of_literal_type(&declared))
                } else {
                    Some(declared)
                }
            }
            crate::ast::NodeData::PropertyAccessExpression(pa) => {
                let obj_type = self.get_type_of_node(&pa.expression);

                self.get_property_of_type(&obj_type, &pa.name.text())
                    .map(|sym| self.write_type_of_property_symbol(&sym))
            }
            crate::ast::NodeData::ElementAccessExpression(ea) => {
                if ea.argument_expression.kind == SyntaxKind::StringLiteral {
                    let obj_type = self.get_type_of_node(&ea.expression);
                    let name = ea.argument_expression.text();
                    if let Some(prop) = self.get_property_of_type(&obj_type, name) {
                        return Some(self.write_type_of_property_symbol(&prop));
                    }
                }
                let obj_type = self.get_type_of_node(&ea.expression);
                let index_type = self.get_type_of_node(&ea.argument_expression);
                Some(self.get_indexed_access_type(&obj_type, &index_type))
            }
            _ => None,
        }
    }

    pub(crate) fn assignment_target_is_readonly(&mut self, target: &Arc<Node>) -> bool {
        match &target.data {
            crate::ast::NodeData::PropertyAccessExpression(pa) => {
                let obj_type = self.get_type_of_node(&pa.expression);
                if let Some(sym) = self.get_property_of_type(&obj_type, &pa.name.text())
                    && (sym.check_flags.contains(crate::ast::CheckFlags::Readonly)
                        || sym
                            .declarations
                            .iter()
                            .any(|d| d.has_syntactic_modifier(ModifierFlags::Readonly)))
                {
                    return true;
                }
                self.namespace_const_member(&pa.expression, &pa.name.text())
                    .is_some()
            }

            crate::ast::NodeData::ElementAccessExpression(ea)
                if ea.argument_expression.kind == SyntaxKind::StringLiteral =>
            {
                self.namespace_const_member(&ea.expression, ea.argument_expression.text())
                    .is_some()
            }
            _ => false,
        }
    }

    pub(crate) fn namespace_const_member(
        &mut self,
        obj_expr: &Arc<Node>,
        name: &str,
    ) -> Option<Arc<crate::ast::Symbol>> {
        if obj_expr.kind != SyntaxKind::Identifier {
            return None;
        }
        let sym = self.resolve_identifier(obj_expr)?;
        let base = self.resolve_alias_base(sym);
        if !base.flags.contains(SymbolFlags::ValueModule) {
            return None;
        }
        let member = base
            .exports
            .get(name)
            .or_else(|| base.members.get(name))
            .cloned()
            .or_else(|| {
                base.declarations
                    .iter()
                    .filter(|d| d.kind == SyntaxKind::ModuleDeclaration)
                    .find_map(|d| {
                        self.program
                            .symbol_map()
                            .locals
                            .get(&d.id())
                            .and_then(|l| l.get(name).cloned())
                    })
            });
        member.filter(|m| self.symbol_is_const_variable(m))
    }
}
