#![allow(unused_imports)]

use crate::checker::checker_classes::*;

// 类型字面量同名属性查重（Go checkObjectTypeForDuplicateDeclarations 的
// TypeLiteral 入口）与合并属性类型一致性（TS2717）
impl Checker {
    pub(crate) fn check_type_literal_duplicate_declarations(&mut self, node: &Arc<Node>) {
        let members: &[Arc<Node>] = match &node.data {
            tsox_frontend::ast::NodeData::TypeLiteralNode(d) => &d.members.nodes,
            _ => return,
        };
        let member_name = |m: &Arc<Node>| -> Option<String> {
            let n = m.name()?;
            match n.kind {
                SyntaxKind::Identifier | SyntaxKind::StringLiteral => Some(n.text().to_string()),
                SyntaxKind::NumericLiteral => {
                    Some(tsox_core::jsnum::Number::from_string(n.text()).to_string())
                }
                _ => None,
            }
        };
        let mut seen: std::collections::HashMap<String, Vec<&Arc<Node>>> =
            std::collections::HashMap::new();
        for m in members.iter() {
            if m.kind == SyntaxKind::PropertySignature
                && let Some(name) = member_name(m)
            {
                seen.entry(name).or_default().push(m);
            }
        }
        for (_, group) in seen.iter() {
            if group.len() > 1 {
                let display = group
                    .first()
                    .and_then(|m| m.name())
                    .and_then(|n| self.node_source_text(&n).or_else(|| Some(n.text().to_string())))
                    .unwrap_or_default();
                for m in group.iter() {
                    let Some(name_node) = m.name() else { continue };
                    self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                        self.current_file.clone(),
                        name_node.loc,
                        tsox_core::diagnostics::messages_generated::DUPLICATE_IDENTIFIER_0,
                        vec![display.clone()],
                    ));
                }
                let later = group.last().copied();
                let first = group.first().copied();
                if let (Some(first), Some(later)) = (first, later) {
                    let first_t =
                        member_type_of(first).map(|tn| self.get_type_from_type_node(&tn));
                    let later_t =
                        member_type_of(later).map(|tn| self.get_type_from_type_node(&tn));
                    if let (Some(f), Some(l)) = (first_t, later_t) {
                        let f_str = self.type_to_string(&f);
                        let l_str = self.type_to_string(&l);
                        if f_str != l_str {
                            let display = later
                                .name()
                                .and_then(|n| {
                                    self.node_source_text(&n)
                                        .or_else(|| Some(n.text().to_string()))
                                })
                                .unwrap_or_default();
                            let loc = later.name().map(|n| n.loc).unwrap_or(later.loc);
                            self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                                self.current_file.clone(),
                                loc,
                                tsox_core::diagnostics::messages_generated::
                                    SUBSEQUENT_PROPERTY_DECLARATIONS_MUST_HAVE_THE_SAME_TYPE_PROPERTY_0_MUST_BE_OF_TYPE_1_BUT_HERE_HAS_TYPE_2,
                                vec![display, f_str, l_str],
                            ));
                        }
                    }
                }
            }
        }
    }

    // Go errorNextVariableOrPropertyDeclarationMustHaveSameType（属性合并后
    // 逐对类型不一致在后续声明处报 TS2717）；按 binder 名配对，计算名不参与
    pub(crate) fn check_class_property_type_consistency(&mut self, members: &[Arc<Node>]) {
        let member_name = |m: &Arc<Node>| -> Option<String> {
            let n = m.name()?;
            match n.kind {
                SyntaxKind::Identifier
                | SyntaxKind::StringLiteral
                | SyntaxKind::PrivateIdentifier => Some(n.text().to_string()),
                SyntaxKind::NumericLiteral => {
                    Some(tsox_core::jsnum::Number::from_string(n.text()).to_string())
                }
                SyntaxKind::ComputedPropertyName => {
                    let tsox_frontend::ast::NodeData::ComputedPropertyName(cd) = &n.data else {
                        return None;
                    };
                    match cd.expression.kind {
                        SyntaxKind::StringLiteral | SyntaxKind::NumericLiteral => {
                            Some(cd.expression.text().to_string())
                        }
                        _ => crate::binder::symbols_binder_4::well_known_symbol_member_name(
                            &cd.expression,
                        ),
                    }
                }
                _ => None,
            }
        };
        for (idx, m) in members.iter().enumerate() {
            if m.kind != SyntaxKind::PropertyDeclaration {
                continue;
            }
            let Some(name) = member_name(m) else { continue };
            let is_static = m.has_syntactic_modifier(ModifierFlags::Static);
            let later_type = self.property_or_accessor_declared_type(m);
            let earlier = members.iter().take(idx).find(|e| {
                matches!(
                    e.kind,
                    SyntaxKind::PropertyDeclaration
                        | SyntaxKind::GetAccessor
                        | SyntaxKind::SetAccessor
                ) && e.has_syntactic_modifier(ModifierFlags::Static) == is_static
                    && member_name(e).is_some_and(|n| n == name)
            });
            let Some(earlier) = earlier else { continue };
            let first_type = self.property_or_accessor_declared_type(earlier);
            let (Some(first_type), Some(later_type)) = (first_type, later_type) else {
                continue;
            };
            let widen = |c: &mut Self, t: &Arc<crate::checker::types::Type>| {
                if crate::checker::is_literal_type(t) {
                    c.get_base_type_of_literal_type(t)
                } else {
                    t.clone()
                }
            };
            let first_type = widen(self, &first_type);
            let later_type = widen(self, &later_type);
            let first_str = self.type_to_string(&first_type);
            let later_str = self.type_to_string(&later_type);
            if first_str != later_str {
                let loc = m.name().map(|n| n.loc).unwrap_or(m.loc);
                self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                    self.current_file.clone(),
                    loc,
                    tsox_core::diagnostics::messages_generated::
                        SUBSEQUENT_PROPERTY_DECLARATIONS_MUST_HAVE_THE_SAME_TYPE_PROPERTY_0_MUST_BE_OF_TYPE_1_BUT_HERE_HAS_TYPE_2,
                    vec![name, first_str, later_str],
                ));
            }
        }
    }

    // Go 属性/访问器合并符号的声明类型：属性取注解或初始化式类型，
    // getter 取返回注解或首个 return 表达式，setter 取参数注解
    fn property_or_accessor_declared_type(
        &mut self,
        m: &Arc<Node>,
    ) -> Option<Arc<crate::checker::types::Type>> {
        match &m.data {
            tsox_frontend::ast::NodeData::PropertyDeclaration(pd) => {
                if let Some(tn) = &pd.type_node {
                    Some(self.get_type_from_type_node(tn))
                } else if let Some(init) = &pd.initializer {
                    Some(self.get_type_of_node(init))
                } else {
                    Some(self.get_any_type())
                }
            }
            tsox_frontend::ast::NodeData::GetAccessorDeclaration(gd) => {
                if let Some(tn) = &gd.type_node {
                    Some(self.get_type_from_type_node(tn))
                } else {
                    Self::first_return_expression(gd.body.as_ref())
                        .map(|e| self.get_type_of_node(&e))
                }
            }
            tsox_frontend::ast::NodeData::SetAccessorDeclaration(sd) => {
                sd.parameters.iter().next().and_then(|p| {
                    if let tsox_frontend::ast::NodeData::ParameterDeclaration(pd) = &p.data {
                        pd.type_node
                            .as_ref()
                            .map(|tn| self.get_type_from_type_node(tn))
                    } else {
                        None
                    }
                })
            }
            _ => None,
        }
    }
}

fn member_type_of(m: &Arc<Node>) -> Option<Arc<Node>> {
    match &m.data {
        tsox_frontend::ast::NodeData::PropertySignatureDeclaration(d) => {
            let tn = Arc::clone(&d.type_node);
            (tn.kind != SyntaxKind::MissingDeclaration).then_some(tn)
        }
        _ => None,
    }
}
