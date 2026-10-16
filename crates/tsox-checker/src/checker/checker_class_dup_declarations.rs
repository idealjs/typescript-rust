#![allow(unused_imports)]

use crate::checker::checker_classes::*;

// Go checkObjectTypeForDuplicateDeclarations：同名属性/访问器组合在
// binder 合并后由 checker 报 Duplicate identifier；计算名经 lateBindMember
// 解析后并入同名组（见 checker_class_dup_declarations_late.rs）；类型
// 一致性 TS2717 见 checker_class_dup_declarations_types.rs
impl Checker {
    // 组内首个匹配成员的原始名（源文本，字符串保留引号）
    fn raw_member_display_name(
        &self,
        m: &Arc<Node>,
        name: &str,
        is_static: bool,
    ) -> Option<String> { ::tsox_core::fntrace::enter("raw_member_display_name"); 
        self_member_name_text(m, name, is_static)
            .and_then(|_| m.name())
            .and_then(|n| self.node_source_text(&n).or_else(|| Some(n.text().to_string())))
    }

    pub(crate) fn check_class_type_for_duplicate_declarations(&mut self, node: &Arc<Node>) {
        let members: &[Arc<Node>] = match &node.data {
            tsox_frontend::ast::NodeData::ClassDeclaration(d) => &d.members.nodes,
            tsox_frontend::ast::NodeData::InterfaceDeclaration(d) => &d.members.nodes,
            _ => return,
        };
        self.check_members_duplicate_declarations(members);
    }

    fn check_members_duplicate_declarations(&mut self, members: &[Arc<Node>]) {
        let is_param_prop = |p: &Arc<Node>| {
            p.syntactic_modifier_flags().intersects(
                ModifierFlags::Public
                    | ModifierFlags::Private
                    | ModifierFlags::Protected
                    | ModifierFlags::Readonly,
            ) && p
                .name()
                .is_some_and(|n| n.kind == SyntaxKind::Identifier)
        };
        let param_props: Vec<(String, &Arc<Node>)> = members
            .iter()
            .filter(|m| m.kind == SyntaxKind::Constructor)
            .flat_map(|ctor| {
                let tsox_frontend::ast::NodeData::ConstructorDeclaration(cd) = &ctor.data else {
                    return Vec::new();
                };
                cd.parameters
                    .iter()
                    .filter(|p| is_param_prop(p))
                    .map(|p| (p.name().unwrap().text().to_string(), p))
                    .collect()
            })
            .collect();

        // Go getResolvedMembersOrExportsOfSymbol：先解析全部成员名
        //（计算名经 checkComputedPropertyName 定型），后续按符号视角查重
        let mut resolved: std::collections::HashMap<u64, Option<(String, bool)>> =
            std::collections::HashMap::new();
        for m in members.iter() {
            if m.kind == SyntaxKind::Constructor {
                continue;
            }
            let name = self.duplicate_member_name(m);
            resolved.insert(m.id(), name);
        }

        self.check_member_late_merge_conflicts(members, &resolved);

        // Go checkPropertyOrAccessor 状态机：kind 1=属性 2=访问器；
        // 第二个属性或属性跟访问器组合时报错（state 3 封口）
        let kind_of = |m: &Arc<Node>| -> Option<u8> {
            match m.kind {
                SyntaxKind::PropertyDeclaration => Some(
                    if m.has_syntactic_modifier(ModifierFlags::Accessor) {
                        2
                    } else {
                        1
                    },
                ),
                SyntaxKind::PropertySignature => Some(1),
                SyntaxKind::GetAccessor | SyntaxKind::SetAccessor => Some(2),
                _ => None,
            }
        };

        let mut states: std::collections::HashMap<(String, bool), u8> =
            std::collections::HashMap::new();
        let mut record = |key: (String, bool), kind: u8,
                          states: &mut std::collections::HashMap<(String, bool), u8>|
         -> bool {
            let state = states.get(&key).copied().unwrap_or(0);
            let hit = state == 1 || (state == 2 && kind != 2);
            states.insert(key, if hit { 3 } else { kind });
            hit
        };

        for m in members.iter() {
            if m.kind == SyntaxKind::Constructor {
                let tsox_frontend::ast::NodeData::ConstructorDeclaration(cd) = &m.data else {
                    continue;
                };
                for p in cd.parameters.iter() {
                    if !is_param_prop(p) {
                        continue;
                    }
                    let name = p.name().unwrap().text().to_string();
                    if record((name.clone(), false), 1, &mut states) {
                        self.report_duplicate_class_member(
                            members,
                            &name,
                            false,
                            &param_props,
                            &resolved,
                        );
                    }
                }
                continue;
            }
            let Some(kind) = kind_of(m) else { continue };
            let Some((name, _)) = resolved.get(&m.id()).cloned().flatten() else {
                continue;
            };
            let is_static = m.has_syntactic_modifier(ModifierFlags::Static);
            if record((name.clone(), is_static), kind, &mut states) {
                self.report_duplicate_class_member(
                    members,
                    &name,
                    is_static,
                    &param_props,
                    &resolved,
                );
            }
        }

        self.check_class_property_type_consistency(members);
    }

    // Go reportDuplicateMemberErrors：对每个解析名匹配的成员报错
    fn report_duplicate_class_member(
        &mut self,
        members: &[Arc<Node>],
        name: &str,
        is_static: bool,
        param_props: &[(String, &Arc<Node>)],
        resolved: &std::collections::HashMap<u64, Option<(String, bool)>>,
    ) {
        // Go symbolToString：合并入早绑定符号取首个声明名字原文，纯晚绑定
        // 符号取计算名原文（`[foo]`）
        let display: String = members
            .iter()
            .filter(|m| m.kind != SyntaxKind::Constructor)
            .find_map(|m| self.raw_member_display_name(m, name, is_static))
            .or_else(|| {
                members.iter().find_map(|m| {
                    let (n, is_late) = resolved.get(&m.id()).cloned().flatten()?;
                    (n == name
                        && is_late
                        && m.has_syntactic_modifier(ModifierFlags::Static) == is_static)
                        .then(|| {
                            m.name()
                                .and_then(|nn| self.node_source_text(&nn))
                                .unwrap_or_else(|| n.clone())
                        })
                })
            })
            .unwrap_or_else(|| name.to_string());
        for m in members.iter() {
            if m.kind == SyntaxKind::Constructor {
                for (pn, p) in param_props {
                    if pn == name {
                        let loc = p.name().map(|n| n.loc).unwrap_or(p.loc);
                        self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                            self.current_file.clone(),
                            loc,
                            tsox_core::diagnostics::messages_generated::DUPLICATE_IDENTIFIER_0,
                            vec![name.to_string()],
                        ));
                    }
                }
                continue;
            }
            let matches = resolved
                .get(&m.id())
                .cloned()
                .flatten()
                .is_some_and(|(n, _)| n == name)
                && m.has_syntactic_modifier(ModifierFlags::Static) == is_static;
            if matches {
                let loc = m.name().map(|n| n.loc).unwrap_or(m.loc);
                self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                    self.current_file.clone(),
                    loc,
                    tsox_core::diagnostics::messages_generated::DUPLICATE_IDENTIFIER_0,
                    vec![display.clone()],
                ));
            }
        }
    }
}

fn self_member_name_text(m: &Arc<Node>, name: &str, is_static: bool) -> Option<String> { ::tsox_core::fntrace::enter("self_member_name_text"); 
    let n = m.name()?;
    let matched = match n.kind {
        SyntaxKind::Identifier | SyntaxKind::StringLiteral => n.text().to_string(),
        SyntaxKind::NumericLiteral => tsox_core::jsnum::Number::from_string(n.text()).to_string(),
        SyntaxKind::ComputedPropertyName => {
            let tsox_frontend::ast::NodeData::ComputedPropertyName(cd) = &n.data else {
                return None;
            };
            match cd.expression.kind {
                SyntaxKind::StringLiteral | SyntaxKind::NumericLiteral => {
                    cd.expression.text().to_string()
                }
                _ => crate::binder::symbols_binder_4::well_known_symbol_member_name(&cd.expression)?,
            }
        }
        _ => return None,
    };
    (matched == name && m.has_syntactic_modifier(ModifierFlags::Static) == is_static)
        .then(|| n.text().to_string())
}
