#![allow(unused_imports)]

use crate::checker::checker_expr_access::*;

impl Checker {
    pub(crate) fn get_type_of_binary_expression(&mut self, node: &Arc<Node>) -> Arc<Type> {
        use tsox_frontend::ast::SyntaxKind::*;
        if let tsox_frontend::ast::NodeData::BinaryExpression(data) = &node.data {
            match data.operator_token.kind {
                PlusToken => {
                    let lt = self.get_type_of_node(&data.left);
                    let rt = self.get_type_of_node(&data.right);
                    let string_like = |t: &Arc<Type>| {
                        t.flags
                            .intersects(TypeFlags::String | TypeFlags::StringLiteral)
                    };
                    if string_like(&lt) || string_like(&rt) {
                        self.string_type()
                    } else if lt.flags.contains(TypeFlags::Any) || rt.flags.contains(TypeFlags::Any)
                    {
                        self.get_any_type()
                    } else {
                        self.number_type()
                    }
                }

                MinusToken
                | AsteriskToken
                | SlashToken
                | PercentToken
                | AsteriskAsteriskToken
                | LessThanLessThanToken
                | GreaterThanGreaterThanToken
                | GreaterThanGreaterThanGreaterThanToken
                | AmpersandToken
                | BarToken
                | CaretToken => self.number_type(),

                LessThanToken
                | GreaterThanToken
                | LessThanEqualsToken
                | GreaterThanEqualsToken
                | EqualsEqualsToken
                | ExclamationEqualsToken
                | EqualsEqualsEqualsToken
                | ExclamationEqualsEqualsToken
                | InKeyword
                | InstanceOfKeyword => self.boolean_type(),

                AmpersandAmpersandToken | AmpersandAmpersandEqualsToken => {
                    // Go checkBinaryExpressionWorker：&& 结果 =
                    // union(extractDefinitelyFalsy(left), right)，
                    // 左操作数无真值事实时结果为左类型
                    let left_type = self.get_type_of_node(&data.left);
                    let right_type = self.get_type_of_node(&data.right);
                    if self
                        .constituent_types(&left_type)
                        .iter()
                        .any(|c| self.has_truthy_fact(c))
                    {
                        let falsy = self.extract_definitely_falsy_constituents(&left_type);
                        self.get_union_type(vec![falsy, right_type])
                    } else {
                        left_type
                    }
                }

                BarBarToken | BarBarEqualsToken => {
                    // Go checkBinaryExpressionWorker：|| 结果 =
                    // union(nonNullable(removeDefinitelyFalsy(left)), right)，
                    // 左操作数无假值事实时结果为左类型
                    let left_type = self.get_type_of_node(&data.left);
                    let right_type = self.get_type_of_node(&data.right);
                    if self
                        .constituent_types(&left_type)
                        .iter()
                        .any(|c| self.has_falsy_fact(c))
                    {
                        let removed = self.remove_definitely_falsy_constituents(&left_type);
                        let non_null = self.get_non_nullable_type_of(&removed);
                        let reduced = self.remove_subtype_redundant_members(vec![
                            non_null,
                            Arc::clone(&right_type),
                        ]);
                        self.get_union_type(reduced)
                    } else {
                        left_type
                    }
                }

                QuestionQuestionToken | QuestionQuestionEqualsToken => {
                    // Go checkBinaryExpressionWorker：?? 结果 =
                    // union(nonNullable(left), right)，左操作数不可空时为左类型
                    let left_type = self.get_type_of_node(&data.left);
                    let right_type = self.get_type_of_node(&data.right);
                    let may_be_nullish = self.constituent_types(&left_type).iter().any(|c| {
                        c.flags.intersects(
                            TypeFlags::Undefined
                                | TypeFlags::Null
                                | TypeFlags::Any
                                | TypeFlags::Unknown
                                | TypeFlags::TypeParameter,
                        )
                    });
                    if may_be_nullish {
                        let non_null = self.get_non_nullable_type_of(&left_type);
                        let reduced =
                            self.remove_subtype_redundant_members(vec![non_null, right_type]);
                        self.get_union_type(reduced)
                    } else {
                        left_type
                    }
                }

                CommaToken => self.get_type_of_node(&data.right),

                EqualsToken => self.get_type_of_node(&data.right),

                PlusEqualsToken => {
                    let left_type = self.get_type_of_node(&data.left);
                    let lt = self.get_base_type_of_literal_type(&left_type);
                    let rt = self.get_type_of_node(&data.right);
                    let string_like = |t: &Arc<Type>| {
                        t.flags
                            .intersects(TypeFlags::String | TypeFlags::StringLiteral)
                    };
                    if string_like(&lt) || string_like(&rt) {
                        self.string_type()
                    } else if lt.flags.contains(TypeFlags::Any) || rt.flags.contains(TypeFlags::Any)
                    {
                        self.get_any_type()
                    } else {
                        self.number_type()
                    }
                }

                MinusEqualsToken
                | AsteriskEqualsToken
                | SlashEqualsToken
                | PercentEqualsToken
                | AsteriskAsteriskEqualsToken
                | LessThanLessThanEqualsToken
                | GreaterThanGreaterThanEqualsToken
                | GreaterThanGreaterThanGreaterThanEqualsToken
                | AmpersandEqualsToken
                | BarEqualsToken
                | CaretEqualsToken => self.number_type(),
                _ => self.get_any_type(),
            }
        } else {
            self.get_any_type()
        }
    }

    pub(crate) fn get_type_of_property_access(&mut self, node: &Arc<Node>) -> Arc<Type> {
        let (obj_expr, name) = match &node.data {
            tsox_frontend::ast::NodeData::PropertyAccessExpression(data) => {
                (&data.expression, &data.name)
            }
            _ => return self.get_any_type(),
        };

        if obj_expr.kind == SyntaxKind::Identifier
            && let Some(sym) = self.resolve_identifier(obj_expr)
        {
            let base = self.resolve_alias_base(sym);
            if base.flags.contains(SymbolFlags::ValueModule) {
                let name_text = name.text();
                let member = base
                    .exports
                    .get(name_text)
                    .or_else(|| base.members.get(name_text))
                    .cloned()
                    .or_else(|| self.ambient_namespace_local(&base, name_text));

                if member.is_none() && !self.ambient_namespace_locals_visible(&base) {
                    return self.error_type();
                }
                if let Some(member) = member {
                    if let Some(t) = self
                        .value_symbol_links
                        .get(&member)
                        .and_then(|l| l.resolved_type.clone())
                    {
                        return t;
                    }
                    for decl in &member.declarations {
                        match decl.kind {
                            SyntaxKind::FunctionDeclaration => {
                                return self.get_type_of_function_like(decl);
                            }
                            SyntaxKind::ClassDeclaration => {
                                return self.get_type_of_class_declaration(decl);
                            }

                            SyntaxKind::ImportEqualsDeclaration => {
                                let t = self.type_of_imported_symbol(&member);
                                let resolved = match t {
                                    Some(t)
                                        if !(t.flags.contains(TypeFlags::Any)
                                            && t.intrinsic_name() == Some("any")) =>
                                    {
                                        Some(t)
                                    }
                                    _ => {
                                        let base = self.resolve_alias_base(Arc::clone(&member));
                                        base.declarations
                                            .iter()
                                            .find(|d| d.kind == SyntaxKind::ClassDeclaration)
                                            .map(|cd| self.get_type_of_class_declaration(cd))
                                    }
                                };
                                if let Some(t) = resolved {
                                    return t;
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        let obj_type = self.get_type_of_node(obj_expr);

        if obj_type.intrinsic_name() == Some("error") {
            return self.error_type();
        }
        let name_text = name.text();

        if obj_type.is_union() {
            let parts: Vec<Arc<Type>> = self
                .constituent_types(&obj_type)
                .into_iter()
                .filter_map(|c| {
                    let sym = self.get_property_of_type(&c, &name_text)?;
                    if let Some(sub) = self.instantiate_array_member_type(&c, &sym) {
                        return Some(sub);
                    }
                    if c.as_object().is_some_and(|o| !o.type_arguments.is_empty()) {
                        return Some(self.substituted_member_type_of(&c, &sym));
                    }
                    Some(self.get_type_of_symbol(&sym))
                })
                .collect();
            if !parts.is_empty() {
                let t = if parts.len() == 1 {
                    parts.into_iter().next().expect("exactly one")
                } else {
                    self.get_union_type(parts)
                };
                return self.flow_type_of_access_expression(node, None, t);
            }
        }

        if (self.is_auto_array_type(&obj_type)
            || obj_type.object_flags.contains(ObjectFlags::EvolvingArray))
            && self.is_array_mutation_method(&name_text)
        {
            return self.get_any_type();
        }
        if let Some(sym) = self.get_property_of_type(&obj_type, &name_text) {
            if let Some(substituted) = self.instantiate_array_member_type(&obj_type, &sym) {
                return self.flow_type_of_access_expression(node, Some(&sym), substituted);
            }

            if obj_type
                .as_object()
                .is_some_and(|o| !o.type_arguments.is_empty())
            {
                let substituted = self.substituted_member_type_of(&obj_type, &sym);
                return self.flow_type_of_access_expression(node, Some(&sym), substituted);
            }
            // 类实例型成员是急建合成符号（无注解方法返回 any 驻缓存）：
            // 回源 binder 声明符号走惰性体推断（this 返回型等）
            let prop_type = self.member_decl_symbol_type(&sym);
            return self.flow_type_of_access_expression(node, Some(&sym), prop_type);
        }

        if name_text == "length" && self.is_array_type(&obj_type) {
            return self.number_type();
        }
        // 无具名成员：适用的字符串索引签名值（tsc getApplicableIndexInfo）
        if let Some(structured) = obj_type.as_structured() {
            for info in &structured.index_infos {
                let key_matches = info.key_type.as_ref().is_some_and(|k| {
                    k.flags.contains(TypeFlags::String)
                        || k.flags.contains(TypeFlags::Number)
                });
                if key_matches {
                    if let Some(value) = &info.value_type {
                        return self.flow_type_of_access_expression(node, None, Arc::clone(value));
                    }
                }
            }
        }
        self.get_any_type()
    }

    pub(crate) fn flow_type_of_access_expression(
        &mut self,
        node: &Arc<Node>,
        prop: Option<&Arc<Symbol>>,
        prop_type: Arc<Type>,
    ) -> Arc<Type> {
        if Self::is_definite_assignment_target(node) {
            return prop_type;
        }
        if let Some(prop) = prop {
            let eligible = prop
                .flags
                .intersects(SymbolFlags::VARIABLE | SymbolFlags::Property | SymbolFlags::ACCESSOR)
                || (prop.flags.contains(SymbolFlags::Method) && prop_type.is_union());
            if !eligible {
                return prop_type;
            }
        }
        self.get_flow_type_of_reference(node, &prop_type)
    }

    pub(crate) fn is_definite_assignment_target(node: &Arc<Node>) -> bool {
        let Some(parent) = &node.parent() else {
            return false;
        };
        match &parent.data {
            NodeData::BinaryExpression(bin) => {
                Self::is_assignment_operator(bin.operator_token.kind)
                    && Arc::ptr_eq(&bin.left, node)
            }
            NodeData::PostfixUnaryExpression(unary) => {
                matches!(
                    unary.operator,
                    SyntaxKind::PlusPlusToken | SyntaxKind::MinusMinusToken
                ) && Arc::ptr_eq(&unary.operand, node)
            }
            NodeData::PrefixUnaryExpression(unary) => {
                matches!(
                    unary.operator,
                    SyntaxKind::PlusPlusToken | SyntaxKind::MinusMinusToken
                ) && Arc::ptr_eq(&unary.operand, node)
            }
            _ => false,
        }
    }

    pub(crate) fn is_assignment_operator(kind: tsox_frontend::ast::SyntaxKind) -> bool {
        use tsox_frontend::ast::SyntaxKind::*;
        matches!(
            kind,
            EqualsToken
                | PlusEqualsToken
                | MinusEqualsToken
                | AsteriskEqualsToken
                | SlashEqualsToken
                | PercentEqualsToken
                | AsteriskAsteriskEqualsToken
                | LessThanLessThanEqualsToken
                | GreaterThanGreaterThanEqualsToken
                | GreaterThanGreaterThanGreaterThanEqualsToken
                | AmpersandEqualsToken
                | BarEqualsToken
                | CaretEqualsToken
                | BarBarEqualsToken
                | AmpersandAmpersandEqualsToken
                | QuestionQuestionEqualsToken
        )
    }

    pub(crate) fn is_block_terminating_statement(stmt: &Arc<Node>) -> bool {
        matches!(
            stmt.kind,
            SyntaxKind::ReturnStatement
                | SyntaxKind::ThrowStatement
                | SyntaxKind::BreakStatement
                | SyntaxKind::ContinueStatement
        )
    }
}
