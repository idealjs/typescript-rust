#![allow(unused_imports)]

use crate::checker::checker_expressions::*;


// 自动编号枚举成员值：首个有显式初值之前的成员按序号，其后递增
fn enum_member_auto_value(member: &Arc<Node>) -> Option<i64> {
    let enum_decl = member.parent()?;
    let members = match &enum_decl.data {
        tsox_frontend::ast::NodeData::EnumDeclaration(d) => &d.members,
        _ => return None,
    };
    let mut next: i64 = 0;
    for m in members.iter() {
        let has_init = match &m.data {
            tsox_frontend::ast::NodeData::EnumMember(em) => em.initializer.is_some(),
            _ => false,
        };
        if Arc::ptr_eq(m, member) {
            return Some(next);
        }
        if has_init {
            if let tsox_frontend::ast::NodeData::EnumMember(em) = &m.data
                && let Some(init) = &em.initializer
                && init.kind == SyntaxKind::NumericLiteral
            {
                next = init.text().parse().unwrap_or(next + 1);
            } else {
                next += 1;
            }
        } else {
            next += 1;
        }
    }
    None
}

impl Checker {
    pub fn check_object_literal_expression(&mut self, node: &Arc<Node>) {
        if let tsox_frontend::ast::NodeData::ObjectLiteralExpression(data) = &node.data {
            let is_destructuring_assignment_target = node.parent().as_ref().is_some_and(|p| match &p
                .data
            {
                tsox_frontend::ast::NodeData::BinaryExpression(b) => {
                    b.operator_token.kind == SyntaxKind::EqualsToken && Arc::ptr_eq(&b.left, node)
                }
                _ => false,
            });
            if is_destructuring_assignment_target
                && self.in_ctor_body_stack.last() == Some(&true)
                && let Some(rhs) = node.parent().as_ref().and_then(|p| match &p.data {
                    tsox_frontend::ast::NodeData::BinaryExpression(b) => Some(Arc::clone(&b.right)),
                    _ => None,
                })
                && rhs.kind == SyntaxKind::ThisKeyword
            {
                let this_type = self.get_type_of_node(&rhs);
                for prop in data.properties.iter() {
                    let Some(name_node) = prop.name() else {
                        continue;
                    };
                    if name_node.kind == SyntaxKind::ComputedPropertyName {
                        continue;
                    }
                    let prop_text = name_node.text().to_string();
                    self.report_abstract_property_access_in_ctor(
                        &name_node, &prop_text, &this_type,
                    );
                }
            }

            if !is_destructuring_assignment_target {
                {
                    let mut seen: std::collections::HashMap<String, Vec<&Arc<Node>>> =
                        std::collections::HashMap::new();
                    for prop in data.properties.iter() {
                        let Some(name_node) = prop.name() else {
                            continue;
                        };
                        let name = if name_node.kind == SyntaxKind::ComputedPropertyName {
                            let expr = match &name_node.data {
                                tsox_frontend::ast::NodeData::ComputedPropertyName(c) => {
                                    Arc::clone(&c.expression)
                                }
                                _ => Arc::clone(name_node),
                            };
                            match expr.kind {
                                // Go getEffectivePropertyNameForPropertyNameNode：
                                // 普通标识符计算键类型非字面量/唯一符号时不可静态定名，
                                // 不参与重复名检测
                                SyntaxKind::NumericLiteral | SyntaxKind::StringLiteral => {
                                    expr.text().to_string()
                                }
                                SyntaxKind::PrefixUnaryExpression => {
                                    let tsox_frontend::ast::NodeData::PrefixUnaryExpression(u) =
                                        &expr.data
                                    else {
                                        continue;
                                    };
                                    let sign = if u.operator == SyntaxKind::MinusToken {
                                        "-"
                                    } else {
                                        ""
                                    };
                                    match &u.operand.data {
                                        tsox_frontend::ast::NodeData::NumericLiteral(n) => {
                                            format!("{sign}{}", n.text)
                                        }
                                        _ => continue,
                                    }
                                }
                                SyntaxKind::PropertyAccessExpression => {
                                    // Go getEffectivePropertyNameForPropertyNameNode：
                                    // `Symbol.<知名符号>` 计算键取内部名参与判重
                                    if let Some(internal) =
                                        crate::binder::symbols_binder_4::well_known_symbol_member_name(&expr)
                                    {
                                        internal
                                    } else {
                                        let sym = self.resolve_qualified_symbol(&expr);
                                        let decl = sym.as_ref().and_then(|s| s.value_declaration.clone());
                                        let Some(decl) = decl else {
                                            continue;
                                        };
                                        if let Some(v) = self.get_constant_value(&decl) {
                                            v
                                        } else if decl.kind == SyntaxKind::EnumMember {
                                            // 自动编号枚举成员按序取值（Go 常量键仍参与判重）
                                            match enum_member_auto_value(&decl) {
                                                Some(v) => v.to_string(),
                                                None => continue,
                                            }
                                        } else if let tsox_frontend::ast::NodeData::VariableDeclaration(vd) =
                                            &decl.data
                                            && let Some(init) = &vd.initializer
                                            && matches!(
                                                init.kind,
                                                SyntaxKind::NumericLiteral | SyntaxKind::StringLiteral
                                            )
                                        {
                                            // 命名空间限定的 const 变量（keys.n）取字面量初值
                                            init.text().to_string()
                                        } else {
                                            continue;
                                        }
                                    }
                                }
                                SyntaxKind::Identifier => {
                                    let sym = self.resolve_identifier(&expr);
                                    let decl = sym.as_ref().and_then(|s| s.value_declaration.clone());
                                    let Some(decl) = decl else {
                                        continue;
                                    };
                                    // const 变量取字面量初值（enum 成员走既有常量通道）
                                    if let tsox_frontend::ast::NodeData::VariableDeclaration(vd) =
                                        &decl.data
                                        && let Some(init) = &vd.initializer
                                        && matches!(
                                            init.kind,
                                            SyntaxKind::NumericLiteral | SyntaxKind::StringLiteral
                                        )
                                    {
                                        init.text().to_string()
                                    } else if let Some(v) = self.get_constant_value(&decl) {
                                        v
                                    } else {
                                        continue;
                                    }
                                }
                                _ => continue,
                            }
                        } else {
                            match name_node.kind {
                                SyntaxKind::StringLiteral
                                | SyntaxKind::NumericLiteral
                                | SyntaxKind::Identifier => name_node.text().to_string(),
                                _ => continue,
                            }
                        };
                        seen.entry(name).or_default().push(prop);
                    }
                    for (_, group) in seen.iter() {
                        let accessor_pair = group.iter().all(|p| {
                            matches!(p.kind, SyntaxKind::GetAccessor | SyntaxKind::SetAccessor)
                        }) && group.len() == 2;
                        if group.len() > 1 && !accessor_pair {
                            let all_accessors = group.iter().all(|p| {
                                matches!(p.kind, SyntaxKind::GetAccessor | SyntaxKind::SetAccessor)
                            });
                            // Go binder：重复访问器走 Duplicate identifier 全员报点，
                            // 且同类访问器重复另报 TS1118（重复的那个上）
                            let mut seen_get = false;
                            let mut seen_set = false;
                            for (i, prop) in group.iter().enumerate() {
                                let Some(name_node) = prop.name() else {
                                    continue;
                                };
                                let name = name_node.text().to_string();
                                let file = self.current_file.clone();
                                if all_accessors {
                                    self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                                        file,
                                        name_node.loc,
                                        tsox_core::diagnostics::messages_generated::DUPLICATE_IDENTIFIER_0,
                                        vec![name.clone()],
                                    ));
                                    let dup_of_kind = match prop.kind {
                                        SyntaxKind::GetAccessor => {
                                            let d = seen_get;
                                            seen_get = true;
                                            d
                                        }
                                        SyntaxKind::SetAccessor => {
                                            let d = seen_set;
                                            seen_set = true;
                                            d
                                        }
                                        _ => false,
                                    };
                                    if dup_of_kind {
                                        self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                                            self.current_file.clone(),
                                            name_node.loc,
                                            tsox_core::diagnostics::messages_generated::
                                                AN_OBJECT_LITERAL_CANNOT_HAVE_MULTIPLE_GET_SLASHSET_ACCESSORS_WITH_THE_SAME_NAME,
                                            vec![name],
                                        ));
                                    }
                                    continue;
                                }
                                if i == 0 {
                                    continue;
                                }
                                self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                                    file,
                                    name_node.loc,
                                    tsox_core::diagnostics::messages_generated::
                                        AN_OBJECT_LITERAL_CANNOT_HAVE_MULTIPLE_PROPERTIES_WITH_THE_SAME_NAME,
                                    vec![name],
                                ));
                            }
                        }
                    }
                }
            }

            for prop in data.properties.iter() {
                let has_setter = data.properties.iter().any(|p| {
                    p.kind == SyntaxKind::SetAccessor
                        && p.name().is_some_and(|n| {
                            n.text() == prop.name().map(|n| n.text()).unwrap_or_default()
                        })
                });
                if prop.kind == SyntaxKind::GetAccessor
                    && !has_setter
                    && self.no_implicit_any
                    && let tsox_frontend::ast::NodeData::GetAccessorDeclaration(gd) = &prop.data
                    && gd.type_node.is_none()
                    && self.getter_return_reaches_this(prop)
                {
                    let name_loc = Self::member_name_node(prop)
                        .map(|n| n.loc)
                        .unwrap_or(prop.loc);
                    let name = Self::member_name_node(prop)
                        .map(|n| n.text().to_string())
                        .unwrap_or_default();
                    let file = self.current_file.clone();
                    self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                            file,
                            name_loc,
                            tsox_core::diagnostics::messages_generated::
                                X_0_IMPLICITLY_HAS_RETURN_TYPE_ANY_BECAUSE_IT_DOES_NOT_HAVE_A_RETURN_TYPE_ANNOTATION_AND_IS_REFERENCED_DIRECTLY_OR_INDIRECTLY_IN_ONE_OF_ITS_RETURN_EXPRESSIONS,
                            vec![name],
                        ));
                }
            }

            let this_typed = self.no_implicit_this
                || self
                    .current_file
                    .as_ref()
                    .is_some_and(|f| f.file_name.ends_with(".js") || f.file_name.ends_with(".jsx"));

            let mut contextual_this: Option<Arc<Type>> = None;
            {
                let mut literal = Arc::clone(node);
                loop {
                    let ctx = self.get_contextual_type(&literal, ContextFlags::None);
                    if let Some(t) = ctx
                        .as_ref()
                        .and_then(|t| self.this_type_marker_argument(t, 0))
                    {
                        contextual_this = Some(t);
                        break;
                    }
                    match &literal.parent().as_ref().map(|p| (p.kind, p.parent())) {
                        Some((SyntaxKind::PropertyAssignment, Some(pp))) => {
                            literal = Arc::clone(pp);
                        }
                        _ => break,
                    }
                }
            }
            let literal_this = match contextual_this {
                Some(t) => t,
                None => self.build_object_literal_this_type(node),
            };
            for prop in data.properties.iter() {
                let method_like = matches!(
                    prop.kind,
                    SyntaxKind::MethodDeclaration
                        | SyntaxKind::GetAccessor
                        | SyntaxKind::SetAccessor
                );

                if let Some(name) = Self::member_name_node(prop) {
                    self.check_computed_property_name(&name);
                }
                if method_like && this_typed {
                    self.this_type_stack.push(Arc::clone(&literal_this));
                }
                self.check_object_literal_element(prop);
                if method_like && this_typed {
                    self.this_type_stack.pop();
                }
            }
        }
    }
}
