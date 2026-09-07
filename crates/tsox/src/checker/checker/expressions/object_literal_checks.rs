#![allow(unused_imports)]

use super::*;

impl Checker {
    pub fn check_object_literal_expression(&mut self, node: &Arc<Node>) {
        if let crate::ast::NodeData::ObjectLiteralExpression(data) = &node.data {
            let is_destructuring_assignment_target = node.parent.as_ref().is_some_and(|p| match &p
                .data
            {
                crate::ast::NodeData::BinaryExpression(b) => {
                    b.operator_token.kind == SyntaxKind::EqualsToken && Arc::ptr_eq(&b.left, node)
                }
                _ => false,
            });
            if is_destructuring_assignment_target
                && self.in_ctor_body_stack.last() == Some(&true)
                && let Some(rhs) = node.parent.as_ref().and_then(|p| match &p.data {
                    crate::ast::NodeData::BinaryExpression(b) => Some(Arc::clone(&b.right)),
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
                                crate::ast::NodeData::ComputedPropertyName(c) => {
                                    Arc::clone(&c.expression)
                                }
                                _ => Arc::clone(name_node),
                            };
                            match expr.kind {
                                SyntaxKind::NumericLiteral
                                | SyntaxKind::StringLiteral
                                | SyntaxKind::Identifier => expr.text().to_string(),
                                SyntaxKind::PrefixUnaryExpression => {
                                    let crate::ast::NodeData::PrefixUnaryExpression(u) = &expr.data
                                    else {
                                        continue;
                                    };
                                    let sign = if u.operator == SyntaxKind::MinusToken {
                                        "-"
                                    } else {
                                        ""
                                    };
                                    match &u.operand.data {
                                        crate::ast::NodeData::NumericLiteral(n) => {
                                            format!("{sign}{}", n.text)
                                        }
                                        _ => continue,
                                    }
                                }
                                SyntaxKind::PropertyAccessExpression => {
                                    let sym = self.resolve_qualified_symbol(&expr);
                                    match sym.as_ref().and_then(|s| s.value_declaration.clone()) {
                                        Some(decl) => match self.get_constant_value(&decl) {
                                            Some(v) => v,
                                            None => continue,
                                        },
                                        None => continue,
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
                            for (i, prop) in group.iter().enumerate() {
                                if i == 0 {
                                    continue;
                                }
                                if let Some(name_node) = prop.name() {
                                    let name = name_node.text().to_string();
                                    let file = self.current_file.clone();
                                    self.diagnostics.add(crate::ast::Diagnostic::new(
                                        file,
                                        name_node.loc,
                                        crate::diagnostics::messages_generated::
                                            AN_OBJECT_LITERAL_CANNOT_HAVE_MULTIPLE_PROPERTIES_WITH_THE_SAME_NAME,
                                        vec![name],
                                    ));
                                }
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
                    && let crate::ast::NodeData::GetAccessorDeclaration(gd) = &prop.data
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
                    self.diagnostics.add(crate::ast::Diagnostic::new(
                            file,
                            name_loc,
                            crate::diagnostics::messages_generated::
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
                    match &literal.parent.as_ref().map(|p| (p.kind, p.parent.clone())) {
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
