#![allow(unused_imports)]

use super::*;

impl Checker {
    pub(crate) fn is_matching_reference(&self, source: &Arc<Node>, target: &Arc<Node>) -> bool {
        match &target.data {
            NodeData::ParenthesizedExpression(p) => {
                return self.is_matching_reference(source, &p.expression);
            }
            NodeData::NonNullExpression(n) => {
                return self.is_matching_reference(source, &n.expression);
            }
            _ => {}
        }
        match target.kind {
            SyntaxKind::BinaryExpression => {
                if let NodeData::BinaryExpression(bin) = &target.data {
                    if is_assignment_operator(bin.operator_token.kind)
                        && self.is_matching_reference(source, &bin.left)
                    {
                        return true;
                    }
                    if bin.operator_token.kind == SyntaxKind::CommaToken
                        && self.is_matching_reference(source, &bin.right)
                    {
                        return true;
                    }
                }
                return false;
            }
            _ => {}
        }
        match source.kind {
            SyntaxKind::BinaryExpression => {
                if let NodeData::BinaryExpression(bin) = &source.data {
                    if bin.operator_token.kind == SyntaxKind::CommaToken {
                        return self.is_matching_reference(&bin.right, target);
                    }
                    if is_assignment_operator(bin.operator_token.kind) {
                        return self.is_matching_reference(&bin.left, target);
                    }
                }
                return false;
            }
            SyntaxKind::Identifier | SyntaxKind::PrivateIdentifier => {
                if target.kind == SyntaxKind::Identifier {
                    return match (
                        self.resolve_identifier(source),
                        self.resolve_identifier(target),
                    ) {
                        (Some(s), Some(t)) => Arc::ptr_eq(&s, &t),
                        _ => false,
                    };
                }

                if matches!(
                    target.kind,
                    SyntaxKind::VariableDeclaration | SyntaxKind::BindingElement
                ) {
                    let Some(source_sym) = self.resolve_identifier(source) else {
                        return false;
                    };
                    let Some(target_sym) = self.program.symbol_map().symbol_of(target).cloned()
                    else {
                        return false;
                    };

                    let source_unwrapped = source_sym
                        .export_symbol
                        .clone()
                        .unwrap_or_else(|| Arc::clone(&source_sym));
                    let target_unwrapped = target_sym.export_symbol.clone().unwrap_or(target_sym);
                    return Arc::ptr_eq(&source_unwrapped, &target_unwrapped);
                }
                false
            }
            SyntaxKind::ThisKeyword | SyntaxKind::SuperKeyword => target.kind == source.kind,
            SyntaxKind::NonNullExpression
            | SyntaxKind::ParenthesizedExpression
            | SyntaxKind::SatisfiesExpression => {
                if let Some(inner) = source.expression() {
                    self.is_matching_reference(&inner, target)
                } else {
                    false
                }
            }
            SyntaxKind::PropertyAccessExpression | SyntaxKind::ElementAccessExpression => {
                if let Some(source_prop_name) = self.get_accessed_property_name(source) {
                    if matches!(
                        target.kind,
                        SyntaxKind::PropertyAccessExpression | SyntaxKind::ElementAccessExpression
                    ) {
                        if let Some(target_prop_name) = self.get_accessed_property_name(target) {
                            if target_prop_name == source_prop_name {
                                let source_receiver = source.expression();
                                let target_receiver = target.expression();
                                if let (Some(s), Some(t)) = (source_receiver, target_receiver) {
                                    return self.is_matching_reference(&s, &t);
                                }
                            }
                        }
                    }
                }

                if source.kind == SyntaxKind::ElementAccessExpression
                    && target.kind == SyntaxKind::ElementAccessExpression
                {
                    let (
                        NodeData::ElementAccessExpression(source_ea),
                        NodeData::ElementAccessExpression(target_ea),
                    ) = (&source.data, &target.data)
                    else {
                        return false;
                    };
                    if source_ea.argument_expression.kind == SyntaxKind::Identifier
                        && target_ea.argument_expression.kind == SyntaxKind::Identifier
                    {
                        let matching_args = match (
                            self.resolve_identifier(&source_ea.argument_expression),
                            self.resolve_identifier(&target_ea.argument_expression),
                        ) {
                            (Some(s), Some(t)) if Arc::ptr_eq(&s, &t) => {
                                self.symbol_is_const_variable(&s)
                                    || (self.is_parameter_or_mutable_local(&s)
                                        && !self.symbol_is_assigned(&s))
                            }
                            _ => false,
                        };
                        if matching_args {
                            let (Some(s), Some(t)) = (source.expression(), target.expression())
                            else {
                                return false;
                            };
                            return self.is_matching_reference(&s, &t);
                        }
                    }
                }
                false
            }
            SyntaxKind::QualifiedName => {
                if let NodeData::QualifiedName(qualified) = &source.data {
                    if matches!(
                        target.kind,
                        SyntaxKind::PropertyAccessExpression | SyntaxKind::ElementAccessExpression
                    ) {
                        if let Some(target_prop_name) = self.get_accessed_property_name(target) {
                            if qualified.right.text() == target_prop_name {
                                if let Some(t) = target.expression() {
                                    return self.is_matching_reference(&qualified.left, &t);
                                }
                            }
                        }
                    }
                }
                false
            }
            _ => false,
        }
    }

    pub(crate) fn contains_matching_reference(
        &self,
        source: &Arc<Node>,
        target: &Arc<Node>,
    ) -> bool {
        let mut source = Arc::clone(source);
        while matches!(
            source.kind,
            SyntaxKind::PropertyAccessExpression | SyntaxKind::ElementAccessExpression
        ) {
            let Some(inner) = source.expression() else {
                break;
            };
            if self.is_matching_reference(inner, target) {
                return true;
            }
            source = Arc::clone(inner);
        }
        false
    }

    pub(crate) fn get_accessed_property_name(&self, access: &Arc<Node>) -> Option<String> {
        match &access.data {
            NodeData::PropertyAccessExpression(pa) => Some(pa.name.text().to_string()),
            NodeData::ElementAccessExpression(ea) => match &ea.argument_expression.data {
                NodeData::StringLiteral(s) => Some(s.text.clone()),
                NodeData::NumericLiteral(n) => Some(n.text.clone()),
                _ => None,
            },
            _ => None,
        }
    }

    pub(crate) fn is_parameter_or_mutable_local(&self, symbol: &Arc<Symbol>) -> bool {
        symbol
            .flags
            .intersects(SymbolFlags::FunctionScopedVariable | SymbolFlags::BlockScopedVariable)
    }

    pub(crate) fn symbol_is_assigned(&self, symbol: &Arc<Symbol>) -> bool {
        let Some(decl) = symbol.value_declaration.as_ref() else {
            return true;
        };
        let Some(container) = Self::enclosing_function_or_source_file(decl) else {
            return true;
        };
        let mut assigned = false;
        Self::scan_assignment_targets(&container, &symbol.name, &mut assigned);
        assigned
    }

    pub(crate) fn enclosing_function_or_source_file(node: &Arc<Node>) -> Option<Arc<Node>> {
        let mut current = Arc::clone(node);
        loop {
            if matches!(
                current.kind,
                SyntaxKind::SourceFile
                    | SyntaxKind::FunctionDeclaration
                    | SyntaxKind::FunctionExpression
                    | SyntaxKind::ArrowFunction
                    | SyntaxKind::MethodDeclaration
                    | SyntaxKind::Constructor
                    | SyntaxKind::GetAccessor
                    | SyntaxKind::SetAccessor
            ) {
                return Some(current);
            }
            current = Arc::clone(current.parent.as_ref()?);
        }
    }

    pub(crate) fn scan_assignment_targets(node: &Arc<Node>, name: &str, assigned: &mut bool) {
        if *assigned {
            return;
        }
        match &node.data {
            NodeData::BinaryExpression(bin) => {
                if is_assignment_operator(bin.operator_token.kind)
                    && bin.left.kind == SyntaxKind::Identifier
                    && bin.left.text() == name
                {
                    *assigned = true;
                    return;
                }
            }
            NodeData::PrefixUnaryExpression(unary) => {
                if matches!(
                    unary.operator,
                    SyntaxKind::PlusPlusToken | SyntaxKind::MinusMinusToken
                ) && unary.operand.kind == SyntaxKind::Identifier
                    && unary.operand.text() == name
                {
                    *assigned = true;
                    return;
                }
            }
            NodeData::PostfixUnaryExpression(unary) => {
                if matches!(
                    unary.operator,
                    SyntaxKind::PlusPlusToken | SyntaxKind::MinusMinusToken
                ) && unary.operand.kind == SyntaxKind::Identifier
                    && unary.operand.text() == name
                {
                    *assigned = true;
                    return;
                }
            }
            _ => {}
        }
        crate::ast::node_data_generated::for_each_child(node, |child| {
            Self::scan_assignment_targets(child, name, assigned);
            *assigned
        });
    }
}
