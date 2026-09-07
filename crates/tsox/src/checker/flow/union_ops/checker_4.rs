#![allow(unused_imports)]

use super::*;

impl Checker {
    pub(crate) fn const_alias_initializer(&self, expr: &Arc<Node>) -> Option<Arc<Node>> {
        if expr.kind != SyntaxKind::Identifier {
            return None;
        }

        let sym = self.resolve_identifier(expr)?;
        if !self.symbol_is_const_variable(&sym) {
            return None;
        }
        let decl = sym.value_declaration.as_ref()?;
        if decl.kind != SyntaxKind::VariableDeclaration {
            return None;
        }
        let NodeData::VariableDeclaration(var_data) = &decl.data else {
            return None;
        };

        if var_data.type_node.is_some() {
            return None;
        }
        let init = var_data.initializer.as_ref()?;
        Some(Self::skip_parentheses(init))
    }

    pub(crate) fn symbol_is_const_variable(&self, symbol: &Arc<Symbol>) -> bool {
        for decl in &symbol.declarations {
            if let Some(parent) = &decl.parent {
                if parent.kind == SyntaxKind::VariableDeclarationList
                    && parent.flags.contains(NodeFlags::Const)
                {
                    return true;
                }
            }
        }
        false
    }

    pub(crate) fn skip_parentheses(node: &Arc<Node>) -> Arc<Node> {
        let mut current = Arc::clone(node);
        loop {
            if let NodeData::ParenthesizedExpression(p) = &current.data {
                current = Arc::clone(&p.expression);
                continue;
            }
            return current;
        }
    }

    pub(crate) fn evolve_array_at_mutation(
        &mut self,
        node: &Arc<Node>,
        pre_type: &Arc<Type>,
        target: &FlowRef,
    ) -> Option<Arc<Type>> {
        let receiver = self.get_array_mutation_receiver(node)?;
        if !self.expr_matches_target(&receiver, target) {
            return None;
        }

        let evolving = if pre_type.object_flags.contains(ObjectFlags::EvolvingArray) {
            Arc::clone(pre_type)
        } else if self.is_auto_array_type(pre_type) {
            self.get_evolving_array_type(self.never_type())
        } else {
            return Some(Arc::clone(pre_type));
        };

        let args = self.get_call_arguments(node);
        let mut arg_types: Vec<Arc<Type>> = Vec::with_capacity(args.len());
        for arg in &args {
            let t = self.get_type_of_node(arg);
            arg_types.push(self.get_widened_type_of_literal(&t));
        }

        let mut evolved = evolving;
        match &node.data {
            NodeData::BinaryExpression(bin) if is_assignment_operator(bin.operator_token.kind) => {
                if let NodeData::ElementAccessExpression(ea) = &bin.left.data {
                    let index_type = self.get_type_of_node(&ea.argument_expression);
                    if index_type
                        .flags
                        .intersects(TypeFlags::Number | TypeFlags::NumberLiteral)
                    {
                        let t = self.get_type_of_node(&bin.right);
                        let widened = self.get_widened_type_of_literal(&t);
                        evolved = self.add_evolving_array_element_type(&evolved, widened);
                    }
                }
            }
            _ => {
                for arg_type in arg_types {
                    evolved = self.add_evolving_array_element_type(&evolved, arg_type);
                }
            }
        }
        Some(evolved)
    }

    pub(crate) fn get_array_mutation_receiver(&self, node: &Arc<Node>) -> Option<Arc<Node>> {
        match &node.data {
            NodeData::CallExpression(call) => {
                if let NodeData::PropertyAccessExpression(prop) = &call.expression.data {
                    return Some(Arc::clone(&prop.expression));
                }
                None
            }
            NodeData::BinaryExpression(bin) => {
                if let NodeData::ElementAccessExpression(ea) = &bin.left.data {
                    return Some(Arc::clone(&ea.expression));
                }
                None
            }
            _ => None,
        }
    }

    pub(crate) fn get_call_arguments(&self, node: &Arc<Node>) -> Vec<Arc<Node>> {
        match &node.data {
            NodeData::CallExpression(call) => call.arguments.iter().cloned().collect(),
            _ => Vec::new(),
        }
    }

    pub(crate) fn binding_element_in_var_pattern(element: &Arc<Node>) -> bool {
        let pattern = element.parent.as_ref();
        let Some(decl) = pattern.and_then(|p| p.parent.as_ref()) else {
            return false;
        };
        if decl.kind != SyntaxKind::VariableDeclaration {
            return false;
        }
        let Some(list) = decl.parent.as_ref() else {
            return false;
        };
        if list.kind != SyntaxKind::VariableDeclarationList {
            return false;
        }

        !(list
            .flags
            .intersects(crate::ast::node_flags::NodeFlags::Let)
            || list
                .flags
                .intersects(crate::ast::node_flags::NodeFlags::Const))
    }

    pub(crate) fn assignment_flow_type(
        &mut self,
        expr: &Arc<Node>,
        target: &FlowRef,
        declared: &Arc<Type>,
    ) -> Option<Arc<Type>> {
        let evolving = declared.object_flags.contains(ObjectFlags::EvolvingArray)
            || self.is_auto_array_type(declared);
        match &expr.data {
            NodeData::BinaryExpression(bin) => {
                if !is_assignment_operator(bin.operator_token.kind) {
                    return None;
                }
                if !self.expr_matches_target(&bin.left, target) {
                    return None;
                }

                if bin.operator_token.kind == SyntaxKind::EqualsToken {
                    let assigned = if matches!(
                        &bin.right.data,
                        NodeData::ArrayLiteralExpression(d) if d.elements.is_empty()
                    ) {
                        self.auto_array_type()
                    } else {
                        self.get_type_of_node(&bin.right)
                    };
                    return Some(self.reduced_assignment_type(declared, &assigned, evolving));
                }

                let assigned = self.get_type_of_node(&bin.right);
                let possibly_nullish = self
                    .constituent_types(declared)
                    .iter()
                    .any(|c| c.flags.intersects(TypeFlags::Undefined | TypeFlags::Null));
                let possibly_falsy = self
                    .constituent_types(declared)
                    .iter()
                    .any(|c| self.constituent_is_definitely_falsy(c));
                let possibly_truthy = self
                    .constituent_types(declared)
                    .iter()
                    .any(|c| !self.constituent_is_definitely_falsy(c));
                match bin.operator_token.kind {
                    SyntaxKind::QuestionQuestionEqualsToken if possibly_nullish => {
                        let non_null = self.get_non_nullable_type_of(declared);
                        Some(self.flow_union_of(&[non_null, assigned]))
                    }
                    SyntaxKind::BarBarEqualsToken if possibly_falsy => {
                        let truthy = self.remove_definitely_falsy_constituents(declared);
                        Some(self.flow_union_of(&[truthy, assigned]))
                    }
                    SyntaxKind::AmpersandAmpersandEqualsToken if possibly_truthy => {
                        let falsy = self.extract_definitely_falsy_constituents(declared);
                        Some(self.flow_union_of(&[falsy, assigned]))
                    }
                    SyntaxKind::QuestionQuestionEqualsToken
                    | SyntaxKind::BarBarEqualsToken
                    | SyntaxKind::AmpersandAmpersandEqualsToken => Some(Arc::clone(declared)),
                    _ => None,
                }
            }

            NodeData::PostfixUnaryExpression(unary) => {
                if matches!(
                    unary.operator,
                    SyntaxKind::PlusPlusToken | SyntaxKind::MinusMinusToken
                ) && self.expr_matches_target(&unary.operand, target)
                {
                    Some(self.number_type())
                } else {
                    None
                }
            }
            NodeData::PrefixUnaryExpression(unary) => {
                if matches!(
                    unary.operator,
                    SyntaxKind::PlusPlusToken | SyntaxKind::MinusMinusToken
                ) && self.expr_matches_target(&unary.operand, target)
                {
                    Some(self.number_type())
                } else {
                    None
                }
            }

            NodeData::VariableDeclaration(_) | NodeData::BindingElement(_) => {
                let FlowRef::Symbol(symbol) = target else {
                    return None;
                };
                let element_symbol = self.program.symbol_map().symbol_of(expr).cloned();
                let matched = match &element_symbol {
                    Some(s) => {
                        Arc::ptr_eq(s, symbol)
                            || symbol
                                .export_symbol
                                .as_ref()
                                .is_some_and(|e| Arc::ptr_eq(s, e))
                    }

                    None => match &expr.data {
                        NodeData::BindingElement(be) => be
                            .name
                            .as_ref()
                            .and_then(|name| self.resolve_identifier(name))
                            .is_some_and(|s| Arc::ptr_eq(&s, symbol)),
                        _ => false,
                    },
                } || (element_symbol.as_ref().is_some_and(|s| {
                    s.name == symbol.name
                        && symbol
                            .flags
                            .contains(crate::ast::SymbolFlags::FunctionScopedVariable)
                        && Self::binding_element_in_var_pattern(expr)
                }));
                if !matched {
                    return None;
                }
                let assigned = self.initial_type_of_declaration(expr)?;
                Some(self.reduced_assignment_type(declared, &assigned, evolving))
            }

            NodeData::Identifier(_) if self.expr_matches_target(expr, target) => {
                Some(Arc::clone(declared))
            }
            _ => None,
        }
    }
}
