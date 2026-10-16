#![allow(unused_imports)]

use crate::checker::flow_union_ops::*;
use tsox_frontend::ast::FlowNode;

impl Checker {
    pub(crate) fn const_alias_initializer(&self, expr: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("const_alias_initializer"); 
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

    pub(crate) fn symbol_is_const_variable(&self, symbol: &Arc<Symbol>) -> bool { ::tsox_core::fntrace::enter("symbol_is_const_variable"); 
        for decl in &symbol.declarations {
            if let Some(parent) = decl.parent() {
                if parent.kind == SyntaxKind::VariableDeclarationList
                    && parent.flags.contains(NodeFlags::Const)
                {
                    return true;
                }
            }
        }
        false
    }

    pub(crate) fn skip_parentheses(node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("skip_parentheses"); 
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
    ) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("evolve_array_at_mutation"); 
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

    pub(crate) fn get_array_mutation_receiver(&self, node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("get_array_mutation_receiver"); 
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

    pub(crate) fn get_call_arguments(&self, node: &Arc<Node>) -> Vec<Arc<Node>> { ::tsox_core::fntrace::enter("get_call_arguments"); 
        match &node.data {
            NodeData::CallExpression(call) => call.arguments.iter().cloned().collect(),
            _ => Vec::new(),
        }
    }

    pub(crate) fn binding_element_in_var_pattern(element: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("binding_element_in_var_pattern"); 
        let pattern = element.parent();
        let Some(decl) = pattern.and_then(|p| p.parent()) else {
            return false;
        };
        if decl.kind != SyntaxKind::VariableDeclaration {
            return false;
        }
        let Some(list) = decl.parent() else {
            return false;
        };
        if list.kind != SyntaxKind::VariableDeclarationList {
            return false;
        }

        !(list
            .flags
            .intersects(tsox_frontend::ast::node_flags::NodeFlags::Let)
            || list
                .flags
                .intersects(tsox_frontend::ast::node_flags::NodeFlags::Const))
    }

    pub(crate) fn assignment_widens_target(&mut self, expr: &Arc<Node>, target: &FlowRef) -> bool { ::tsox_core::fntrace::enter("assignment_widens_target");
        match &expr.data {
            NodeData::BinaryExpression(bin) => {
                is_assignment_operator(bin.operator_token.kind)
                    && !matches!(
                        bin.operator_token.kind,
                        SyntaxKind::EqualsToken
                            | SyntaxKind::AmpersandAmpersandEqualsToken
                            | SyntaxKind::BarBarEqualsToken
                            | SyntaxKind::QuestionQuestionEqualsToken
                    )
                    && self.expr_matches_target(&bin.left, target)
            }
            NodeData::PostfixUnaryExpression(unary) => {
                matches!(
                    unary.operator,
                    SyntaxKind::PlusPlusToken | SyntaxKind::MinusMinusToken
                ) && self.expr_matches_target(&unary.operand, target)
            }
            NodeData::PrefixUnaryExpression(unary) => {
                matches!(
                    unary.operator,
                    SyntaxKind::PlusPlusToken | SyntaxKind::MinusMinusToken
                ) && self.expr_matches_target(&unary.operand, target)
            }
            _ => false,
        }
    }

    pub(crate) fn assignment_flow_type(
        &mut self,
        expr: &Arc<Node>,
        target: &FlowRef,
        declared: &Arc<Type>,
        flow: &Arc<FlowNode>,
    ) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("assignment_flow_type");
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
                    let declared_for_reduce = if crate::checker::utilities_get_assignment_target::is_in_compound_like_assignment(&bin.left) {
                        self.get_base_type_of_literal_type(declared)
                    } else {
                        Arc::clone(declared)
                    };
                    let assigned = if matches!(
                        &bin.right.data,
                        NodeData::ArrayLiteralExpression(d) if d.elements.is_empty()
                    ) {
                        self.auto_array_type()
                    } else {
                        self.get_type_of_node(&bin.right)
                    };
                    return Some(self.reduced_assignment_type(&declared_for_reduce, &assigned, evolving));
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
                let matched = match target {
                    FlowRef::Symbol(symbol) => {
                        let element_symbol = self.program.symbol_map().symbol_of(expr).cloned();
                        let base = match &element_symbol {
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
                        };
                        base || (element_symbol.as_ref().is_some_and(|s| {
                            s.name == symbol.name
                                && symbol
                                    .flags
                                    .contains(tsox_frontend::ast::SymbolFlags::FunctionScopedVariable)
                                && Self::binding_element_in_var_pattern(expr)
                        }))
                    }
                    FlowRef::Node(source) => {
                        // Go isMatchingReference（flow.go:1680-1683）：标识符引用 vs
                        // 声明节点按符号判等——getExportSymbolOfValueSymbolIfExported(
                        // getResolvedSymbol(source)) == getSymbolOfDeclaration(target)，
                        // 引用侧/声明侧导出符号互指同样命中
                        let source_symbol = self.get_resolved_symbol(source);
                        let Some(source_symbol) = source_symbol else {
                            return None;
                        };
                        let normalized =
                            self.get_export_symbol_of_value_symbol_if_exported(&source_symbol);
                        let decl_symbol = self.get_symbol_of_declaration(expr);
                        let Some(decl_symbol) = decl_symbol else {
                            return None;
                        };
                        Arc::ptr_eq(&normalized, &decl_symbol)
                            || decl_symbol
                                .export_symbol
                                .as_ref()
                                .is_some_and(|e| Arc::ptr_eq(e, &normalized))
                            || normalized
                                .export_symbol
                                .as_ref()
                                .is_some_and(|e| Arc::ptr_eq(e, &decl_symbol))
                    }
                };
                if !matched {
                    return None;
                }
                // Go getTypeAtFlowAssignment（flow.go:244-246）：命中赋值先过
                // isReachableFlowNode，不可达返回 unreachableNever，由
                // getFlowTypeOfReferenceEx 还原为声明型（do-while(true) 之后的
                // 文件级流图整链不可达，声明不窄化）
                if !self.is_reachable_flow_node(flow) {
                    return None;
                }
                // Go getInitialOrAssignedType（flow.go:289-293）→
                // getInitialTypeOfVariableDeclaration（flow.go:2342-2350）取
                // 初始化式原始型（getTypeOfInitializer），不过
                // widenTypeInferredFromInitializer——let x = true 的流赋值窄化
                // 依赖原始字面量型（checker.go:1013 boolean 即 union{false,true}）
                let assigned = self.initial_type_of_declaration_ex(expr, false)?;
                Some(self.reduced_assignment_type(declared, &assigned, evolving))
            }

            NodeData::Identifier(_) if self.expr_matches_target(expr, target) => {
                Some(Arc::clone(declared))
            }
            _ => None,
        }
    }
}
