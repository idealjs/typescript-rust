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

            if !is_destructuring_assignment_target
                && !self
                    .current_file
                    .as_ref()
                    .is_some_and(|f| f.has_parse_diagnostics)
            {
                self.check_grammar_object_literal_member_modifiers(&data.properties.nodes);
                {
                    // Go checkGrammarObjectLiteralExpression：按声明顺序
                    // 记录首个 kind，后续按组合报 2300/1117/1118/1119，
                    // 1118/1119 后停止该对象字面量的后续检查
                    const MEANING_METHOD: u8 = 1;
                    const MEANING_PROPERTY: u8 = 2;
                    const MEANING_GET: u8 = 4;
                    const MEANING_SET: u8 = 8;
                    const MEANING_ACCESSORS: u8 = MEANING_GET | MEANING_SET;
                    let parse_errors = self
                        .current_file
                        .as_ref()
                        .is_some_and(|f| f.has_parse_diagnostics);
                    let mut seen_kinds: std::collections::HashMap<String, u8> =
                        std::collections::HashMap::new();
                    for prop in data.properties.iter() {
                        let Some(name_node) = prop.name() else { continue };
                        let current_kind = match prop.kind {
                            SyntaxKind::PropertyAssignment
                            | SyntaxKind::ShorthandPropertyAssignment => MEANING_PROPERTY,
                            SyntaxKind::MethodDeclaration => MEANING_METHOD,
                            SyntaxKind::GetAccessor => MEANING_GET,
                            SyntaxKind::SetAccessor => MEANING_SET,
                            _ => continue,
                        };
                        let key = self.object_literal_member_key(&name_node);
                        let Some(key) = key else { continue };
                        let existing_kind = seen_kinds.get(&key).copied().unwrap_or(0);
                        if existing_kind == 0 {
                            seen_kinds.insert(key, current_kind);
                            continue;
                        }
                        let raw_name = self
                            .node_source_text(&name_node)
                            .unwrap_or_else(|| name_node.text().to_string());
                        let mut stop = false;
                        if current_kind & MEANING_METHOD != 0
                            && existing_kind & MEANING_METHOD != 0
                        {
                            if !parse_errors {
                                self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                                    self.current_file.clone(),
                                    name_node.loc,
                                    tsox_core::diagnostics::messages_generated::DUPLICATE_IDENTIFIER_0,
                                    vec![raw_name],
                                ));
                            }
                        } else if current_kind & MEANING_PROPERTY != 0
                            && existing_kind & MEANING_PROPERTY != 0
                        {
                            if !parse_errors {
                                self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                                    self.current_file.clone(),
                                    name_node.loc,
                                    tsox_core::diagnostics::messages_generated::
                                        AN_OBJECT_LITERAL_CANNOT_HAVE_MULTIPLE_PROPERTIES_WITH_THE_SAME_NAME,
                                    vec![raw_name],
                                ));
                            }
                        } else if current_kind & MEANING_ACCESSORS != 0
                            && existing_kind & MEANING_ACCESSORS != 0
                        {
                            if existing_kind != MEANING_ACCESSORS
                                && current_kind != existing_kind
                            {
                                seen_kinds.insert(key, current_kind | existing_kind);
                            } else {
                                if !parse_errors {
                                    self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                                        self.current_file.clone(),
                                        name_node.loc,
                                        tsox_core::diagnostics::messages_generated::
                                            AN_OBJECT_LITERAL_CANNOT_HAVE_MULTIPLE_GET_SLASHSET_ACCESSORS_WITH_THE_SAME_NAME,
                                        vec![],
                                    ));
                                }
                                stop = true;
                            }
                        } else if !parse_errors {
                            self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                                self.current_file.clone(),
                                name_node.loc,
                                tsox_core::diagnostics::messages_generated::
                                    AN_OBJECT_LITERAL_CANNOT_HAVE_PROPERTY_AND_ACCESSOR_WITH_THE_SAME_NAME,
                                vec![],
                            ));
                            stop = true;
                        }
                        if stop {
                            break;
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

            self.check_object_literal_spread_overrides(node);

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

impl Checker {
    // Go getEffectivePropertyNameForPropertyNameNode：对象字面量成员的
    // 静态定名键（常量折叠计算键、字面量键），不可定名返回 None
    fn object_literal_member_key(&mut self, name_node: &Arc<Node>) -> Option<String> {
        if name_node.kind == SyntaxKind::ComputedPropertyName {
            let expr = match &name_node.data {
                tsox_frontend::ast::NodeData::ComputedPropertyName(c) => Arc::clone(&c.expression),
                _ => Arc::clone(name_node),
            };
            match expr.kind {
                SyntaxKind::NumericLiteral | SyntaxKind::StringLiteral => {
                    Some(expr.text().to_string())
                }
                SyntaxKind::PrefixUnaryExpression => {
                    let tsox_frontend::ast::NodeData::PrefixUnaryExpression(u) = &expr.data else {
                        return None;
                    };
                    let sign = if u.operator == SyntaxKind::MinusToken { "-" } else { "" };
                    match &u.operand.data {
                        tsox_frontend::ast::NodeData::NumericLiteral(n) => {
                            Some(format!("{sign}{}", n.text))
                        }
                        _ => None,
                    }
                }
                SyntaxKind::PropertyAccessExpression => {
                    if let Some(internal) =
                        crate::binder::symbols_binder_4::well_known_symbol_member_name(&expr)
                    {
                        return Some(internal);
                    }
                    let sym = self.resolve_qualified_symbol(&expr);
                    let decl = sym.as_ref().and_then(|s| s.value_declaration.clone());
                    let Some(decl) = decl else { return None };
                    if let Some(v) = self.get_constant_value(&decl) {
                        Some(v)
                    } else if decl.kind == SyntaxKind::EnumMember {
                        enum_member_auto_value(&decl).map(|v| v.to_string())
                    } else if let tsox_frontend::ast::NodeData::VariableDeclaration(vd) = &decl.data
                        && let Some(init) = &vd.initializer
                        && matches!(
                            init.kind,
                            SyntaxKind::NumericLiteral | SyntaxKind::StringLiteral
                        )
                    {
                        Some(init.text().to_string())
                    } else {
                        None
                    }
                }
                SyntaxKind::Identifier => {
                    let sym = self.resolve_identifier(&expr);
                    let decl = sym.as_ref().and_then(|s| s.value_declaration.clone());
                    let Some(decl) = decl else { return None };
                    if let tsox_frontend::ast::NodeData::VariableDeclaration(vd) = &decl.data
                        && let Some(init) = &vd.initializer
                        && matches!(
                            init.kind,
                            SyntaxKind::NumericLiteral | SyntaxKind::StringLiteral
                        )
                    {
                        Some(init.text().to_string())
                    } else {
                        self.get_constant_value(&decl)
                    }
                }
                _ => None,
            }
        } else {
            match name_node.kind {
                SyntaxKind::NumericLiteral => Some(
                    tsox_core::jsnum::Number::from_string(name_node.text()).to_string(),
                ),
                SyntaxKind::StringLiteral | SyntaxKind::Identifier => {
                    Some(name_node.text().to_string())
                }
                _ => None,
            }
        }
    }
}

