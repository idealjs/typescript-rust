#![allow(unused_imports)]

use crate::checker::checker_assertions_interfaces::*;

impl Checker {
    pub(crate) fn check_assertion_overlap(
        &mut self,
        node: &Arc<Node>,
        expr: &Arc<Node>,
        type_node: &Arc<Node>,
    ) {
        if type_node.kind == SyntaxKind::TypeReference && type_node.text() == "const" {
            return;
        }
        let expr_type = self.get_type_of_node(expr);
        let target_type = self.get_type_from_type_node(type_node);
        let error_type = self.error_type();
        let exempt = |t: &Arc<Type>| {
            Arc::ptr_eq(t, &error_type)
                || t.flags.contains(TypeFlags::Any)
                || t.flags.contains(TypeFlags::Unknown)
                || t.flags.contains(TypeFlags::Never)
        };
        if exempt(&expr_type) || exempt(&target_type) {
            return;
        }
        let expr_base = if crate::checker::is_literal_type(&expr_type) {
            self.get_base_type_of_literal_type(&expr_type)
        } else {
            expr_type
        };
        // Go checkAssertionDeferred：可比性判定用 getRegularTypeOfObjectLiteral
        // （剥 FreshLiteral）+ getWidenedType 后的类型，断言路径不做多余属性检查
        let regular = self.get_regular_type_of_object_literal(&expr_base);
        let widened = self.get_widened_type(&regular);

        let comparable = self.is_type_comparable_to(&widened, &target_type)
            || self.is_type_comparable_to(&target_type, &widened);
        if !comparable {
            let source_str = self.type_to_string(&expr_base);
            let target_str = self.type_to_string(&target_type);
            let file = self.current_file.clone();
            let diag = tsox_frontend::ast::Diagnostic::new(
                file,
                node.loc,
                tsox_core::diagnostics::messages_generated::
                    CONVERSION_OF_TYPE_0_TO_TYPE_1_MAY_BE_A_MISTAKE_BECAUSE_NEITHER_TYPE_SUFFICIENTLY_OVERLAPS_WITH_THE_OTHER_IF_THIS_WAS_INTENTIONAL_CONVERT_THE_EXPRESSION_TO_UNKNOWN_FIRST,
                vec![source_str, target_str],
            );

            if let Some((prop_loc, prop_name, elem_target_str)) =
                self.assertion_excess_detail(&expr, &expr_base, &target_type)
            {
                // Go 基线：excess 属性错误单独呈现（无 2352 转换头）
                let file = self.current_file.clone();
                self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                    file,
                    prop_loc,
                    tsox_core::diagnostics::messages_generated::
                        OBJECT_LITERAL_MAY_ONLY_SPECIFY_KNOWN_PROPERTIES_AND_0_DOES_NOT_EXIST_IN_TYPE_1,
                    vec![prop_name, elem_target_str],
                ));
                return;
            }
            self.diagnostics.add(diag);
        }
    }

    pub(crate) fn assertion_excess_detail(
        &mut self,
        expr: &Arc<Node>,
        expr_type: &Arc<Type>,
        target_type: &Arc<Type>,
    ) -> Option<(TextRange, String, String)> {
        let (elem_source, elem_target, literal_node) = match &expr.data {
            NodeData::ObjectLiteralExpression(_) => (
                Arc::clone(expr_type),
                Arc::clone(target_type),
                Arc::clone(expr),
            ),
            NodeData::ArrayLiteralExpression(d) => {
                let first_obj = d
                    .elements
                    .iter()
                    .find(|e| matches!(&e.data, NodeData::ObjectLiteralExpression(_)))?;
                let st = self.element_type_of(expr_type)?;
                let tt = self.element_type_of(target_type)?;
                (st, tt, Arc::clone(first_obj))
            }
            _ => return None,
        };
        let prop_name = self.get_excess_property_name(&elem_source, &elem_target)?;
        let prop_loc = self.find_object_literal_property_name_node(&literal_node, &prop_name)?;
        let elem_target_str = self.type_to_string(&elem_target);
        Some((
            prop_loc,
            crate::checker::property_name_for_display(&prop_name),
            elem_target_str,
        ))
    }

    pub(crate) fn element_type_of(&self, t: &Arc<Type>) -> Option<Arc<Type>> {
        if t.flags.contains(TypeFlags::Object) {
            if let TypeData::Object(obj) = &t.data
                && !obj.type_arguments.is_empty()
            {
                return Some(Arc::clone(&obj.type_arguments[0]));
            }
        }
        None
    }

    pub(crate) fn check_accessor_in_type_context(&mut self, member: &Arc<Node>) {
        let body = match &member.data {
            tsox_frontend::ast::NodeData::GetAccessorDeclaration(d) => d.body.clone(),
            tsox_frontend::ast::NodeData::SetAccessorDeclaration(d) => d.body.clone(),
            _ => return,
        };
        if let Some(body) = body {
            let file = self.current_file.clone();
            self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                file,
                body.loc,
                tsox_core::diagnostics::messages_generated::
                    AN_IMPLEMENTATION_CANNOT_BE_DECLARED_IN_AMBIENT_CONTEXTS,
                vec![],
            ));
        }
    }

    pub(crate) fn is_entity_name_expression(node: &Arc<Node>) -> bool {
        if node.kind == SyntaxKind::Identifier {
            return true;
        }
        if let tsox_frontend::ast::NodeData::PropertyAccessExpression(pa) = &node.data {
            return pa.name.kind == SyntaxKind::Identifier && Self::is_entity_name_expression(&pa.expression);
        }
        false
    }

    pub(crate) fn check_interface_members(&mut self, members: &NodeList) {
        {
            let mut seen: std::collections::HashMap<String, Vec<&Arc<Node>>> =
                std::collections::HashMap::new();
            for member in members.iter() {
                if let Some(name_node) = member.name() {
                    let name = match name_node.kind {
                        SyntaxKind::StringLiteral
                        | SyntaxKind::NumericLiteral
                        | SyntaxKind::Identifier
                        | SyntaxKind::PrivateIdentifier => name_node.text().to_string(),
                        // Go getEffectivePropertyNameForPropertyNameNode：仅字面量
                        // 与 well-known 计算名可作去重键，其余跳过；计算键与恰好
                        // 同文的转义字面量键分属不同命名空间（Go binder 对计算名
                        // 走匿名符号，二者不构成重复）
                        SyntaxKind::ComputedPropertyName => {
                            let key = match &name_node.data {
                                tsox_frontend::ast::NodeData::ComputedPropertyName(cd) => {
                                    crate::binder::symbols_binder_4::well_known_symbol_member_name(
                                        &cd.expression,
                                    )
                                    .map(|internal| format!("c:{internal}"))
                                    .or_else(|| {
                                        if matches!(
                                            cd.expression.kind,
                                            SyntaxKind::StringLiteral
                                                | SyntaxKind::NumericLiteral
                                        ) {
                                            Some(format!("l:{}", cd.expression.text()))
                                        } else {
                                            None
                                        }
                                    })
                                }
                                _ => None,
                            };
                            match key {
                                Some(k) => k,
                                None => continue,
                            }
                        }
                        _ => continue,
                    };
                    seen.entry(name).or_default().push(member);
                }
            }
            for (_, group) in seen.iter() {
                let all_methods = group.iter().all(|m| m.kind == SyntaxKind::MethodSignature);
                let accessor_pair = group
                    .iter()
                    .all(|m| matches!(m.kind, SyntaxKind::GetAccessor | SyntaxKind::SetAccessor))
                    && group.iter().any(|m| m.kind == SyntaxKind::GetAccessor)
                    && group.iter().any(|m| m.kind == SyntaxKind::SetAccessor);
                if group.len() > 1 && !all_methods && !accessor_pair {
                    // Go reportDuplicateMemberErrors：显示名取符号名，字符串字面量
                    // 名的符号名含引号（按组内首个声明取形）
                    let display = group.iter().find_map(|m| {
                        let nn = m.name()?;
                        Some(
                            self.node_source_text(&nn)
                                .unwrap_or_else(|| nn.text().to_string()),
                        )
                    });
                    for m in group {
                        if let Some(name_node) = m.name() {
                            let name = match &name_node.data {
                                tsox_frontend::ast::NodeData::ComputedPropertyName(cd) => {
                                    crate::binder::symbols_binder_4::well_known_symbol_member_name(
                                        &cd.expression,
                                    )
                                    .map(|internal| {
                                        crate::checker::property_name_for_display(&internal)
                                    })
                                    .unwrap_or_else(|| name_node.text().to_string())
                                }
                                _ => display
                                    .clone()
                                    .unwrap_or_else(|| name_node.text().to_string()),
                            };
                            let file = self.current_file.clone();
                            self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                                file,
                                name_node.loc,
                                tsox_core::diagnostics::messages_generated::DUPLICATE_IDENTIFIER_0,
                                vec![name],
                            ));
                        }
                    }
                }
            }
        }
        for member in members.iter() {
            // 接口成员的计算名：解析表达式（TS2304）+
            // Go checkGrammarForInvalidDynamicName（TS1166/1169 族）
            if let Some(name) = Self::member_name_node(member)
                && name.kind == SyntaxKind::ComputedPropertyName
            {
                self.check_computed_property_name(&name);
                self.check_member_dynamic_name_grammar(member);
            }
            // Go checkGrammarModifiers：接口成员的非法修饰符（TS1070 等）
            self.check_grammar_modifiers(member);
            match member.kind {
                SyntaxKind::IndexSignature => {
                    self.check_grammar_index_signature(member);
                }
                SyntaxKind::MethodSignature => {
                    let tsox_frontend::ast::NodeData::MethodSignatureDeclaration(d) = &member.data
                    else {
                        continue;
                    };
                    self.check_parameter_property_modifiers(&d.parameters, false);
                    self.check_parameter_implicit_any(member, &d.parameters, 0);
                    for p in d.parameters.iter() {
                        if let tsox_frontend::ast::NodeData::ParameterDeclaration(pd) = &p.data
                            && let Some(pt) = &pd.type_node
                        {
                            self.check_type_annotation(pt);
                        }
                    }
                    if let Some(tn) = &d.type_node {
                        self.check_type_annotation(tn);
                    }

                    if self.no_implicit_any
                        && d.type_node.is_none()
                        && d.name.kind == SyntaxKind::Identifier
                        && !self
                            .current_file
                            .as_ref()
                            .is_some_and(|f| f.has_parse_diagnostics)
                    {
                        let file = self.current_file.clone();
                        let diagnostic = tsox_frontend::ast::Diagnostic::new(
                            file,
                            d.name.loc,
                            tsox_core::diagnostics::messages_generated::
                                X_0_WHICH_LACKS_RETURN_TYPE_ANNOTATION_IMPLICITLY_HAS_AN_1_RETURN_TYPE,
                            vec![d.name.text().to_string(), "any".to_string()],
                        );
                        self.diagnostics.add(diagnostic);
                    }
                }
                SyntaxKind::ConstructSignature | SyntaxKind::CallSignature => {
                    let (params, type_node) = match &member.data {
                        tsox_frontend::ast::NodeData::ConstructSignatureDeclaration(d) => {
                            (&d.parameters, d.type_node.as_ref())
                        }
                        tsox_frontend::ast::NodeData::CallSignatureDeclaration(d) => {
                            (&d.parameters, d.type_node.as_ref())
                        }
                        _ => continue,
                    };
                    self.check_parameter_property_modifiers(params, false);
                    self.check_parameter_implicit_any(member, params, 0);
                    for p in params.iter() {
                        if let tsox_frontend::ast::NodeData::ParameterDeclaration(pd) = &p.data
                            && let Some(pt) = &pd.type_node
                        {
                            self.check_type_annotation(pt);
                        }
                    }
                    if let Some(tn) = type_node {
                        self.check_type_annotation(tn);
                    }

                    if self.no_implicit_any && type_node.is_none() {
                        let message = if member.kind == SyntaxKind::ConstructSignature {
                            tsox_core::diagnostics::messages_generated::
                                CONSTRUCT_SIGNATURE_WHICH_LACKS_RETURN_TYPE_ANNOTATION_IMPLICITLY_HAS_AN_ANY_RETURN_TYPE
                        } else {
                            tsox_core::diagnostics::messages_generated::
                                CALL_SIGNATURE_WHICH_LACKS_RETURN_TYPE_ANNOTATION_IMPLICITLY_HAS_AN_ANY_RETURN_TYPE
                        };
                        let file = self.current_file.clone();
                        let diagnostic =
                            tsox_frontend::ast::Diagnostic::new(file, member.loc, message, vec![]);
                        self.diagnostics.add(diagnostic);
                    }
                }
                SyntaxKind::PropertySignature => {
                    if let tsox_frontend::ast::NodeData::PropertySignatureDeclaration(d) =
                        &member.data
                    {
                        self.check_type_annotation(&d.type_node);
                        if d.type_node.kind == SyntaxKind::MissingDeclaration && self.no_implicit_any {
                            self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                                self.current_file.clone(),
                                member.loc,
                                tsox_core::diagnostics::messages_generated::
                                    MEMBER_0_IMPLICITLY_HAS_AN_1_TYPE,
                                vec![member.name().map(|n| n.text().to_string()).unwrap_or_default(), "any".to_string()],
                            ));
                        }
                    }
                }

                SyntaxKind::GetAccessor | SyntaxKind::SetAccessor => {
                    self.check_accessor_in_type_context(member);
                }
                _ => {}
            }
        }
    }
}
