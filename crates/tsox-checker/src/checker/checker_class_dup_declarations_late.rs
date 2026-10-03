#![allow(unused_imports)]

use crate::checker::checker_classes::*;

// Go lateBindMember + combineSymbolTables/mergeSymbol 在类/接口成员查重上的
// 语义：late-bindable 计算名解析、早/晚绑定符号 flags 冲突报错（带 related）
// Go lateBindMember：late-bindable 计算名（entity-name 表达式且类型为
// 字符串/数字字面量或 unique symbol）解析出成员名
pub(crate) fn late_bound_computed_member_name(
    checker: &mut Checker,
    m: &Arc<Node>,
) -> Option<String> {
    let name = m.name()?;
    if name.kind != SyntaxKind::ComputedPropertyName {
        return None;
    }
    if !crate::checker::mig::wc3_3::is_late_bindable_ast(&name) {
        return None;
    }
    let t = checker.check_computed_property_name_type(&name);
    if !crate::checker::utilities_token_is_identifier_or_keyword::is_type_usable_as_property_name(
        &t,
    ) {
        return None;
    }
    let resolved =
        crate::checker::utilities_token_is_identifier_or_keyword::get_property_name_from_type(&t);
    (!resolved.is_empty()).then_some(resolved)
}

// Go binder bindWorker 各成员 kind 的符号 includes
fn member_symbol_includes(m: &Arc<Node>) -> Option<SymbolFlags> {
    match m.kind {
        SyntaxKind::PropertyDeclaration => Some(
            if m.has_syntactic_modifier(ModifierFlags::Accessor) {
                SymbolFlags::ACCESSOR
            } else {
                SymbolFlags::Property
            },
        ),
        SyntaxKind::PropertySignature => Some(SymbolFlags::Property),
        SyntaxKind::GetAccessor => Some(SymbolFlags::GetAccessor),
        SyntaxKind::SetAccessor => Some(SymbolFlags::SetAccessor),
        SyntaxKind::MethodDeclaration | SyntaxKind::MethodSignature => Some(SymbolFlags::Method),
        _ => None,
    }
}

impl Checker {
    // Go getSymbolOfDeclaration 的名字等价：字面量/知名符号计算名取 binder 名，
    // 其余计算名按 lateBindMember 解析（含字面量 const、unique symbol）；
    // is_late 标记名字是否来自计算名解析
    pub(crate) fn duplicate_member_name(&mut self, m: &Arc<Node>) -> Option<(String, bool)> {
        let n = m.name()?;
        match n.kind {
            SyntaxKind::Identifier
            | SyntaxKind::StringLiteral
            | SyntaxKind::PrivateIdentifier => Some((n.text().to_string(), false)),
            SyntaxKind::NumericLiteral => Some((
                tsox_core::jsnum::Number::from_string(n.text()).to_string(),
                false,
            )),
            SyntaxKind::ComputedPropertyName => {
                let tsox_frontend::ast::NodeData::ComputedPropertyName(cd) = &n.data else {
                    return None;
                };
                match cd.expression.kind {
                    SyntaxKind::StringLiteral | SyntaxKind::NumericLiteral => {
                        Some((cd.expression.text().to_string(), false))
                    }
                    _ => {
                        if let Some(wk) =
                            crate::binder::symbols_binder_4::well_known_symbol_member_name(
                                &cd.expression,
                            )
                        {
                            return Some((wk, false));
                        }
                        late_bound_computed_member_name(self, m).map(|resolved| (resolved, true))
                    }
                }
            }
            _ => None,
        }
    }

    // Go combineSymbolTables → mergeSymbol：同名早绑定符号与晚绑定符号
    // flags 冲突时 reportMergeSymbolError（带 related）；晚绑定内部冲突走
    // lateBindMember 的就地报错
    pub(crate) fn check_member_late_merge_conflicts(
        &mut self,
        members: &[Arc<Node>],
        resolved: &std::collections::HashMap<u64, Option<(String, bool)>>,
    ) {
        struct Group<'a> {
            early_flags: Option<SymbolFlags>,
            early_decls: Vec<&'a Arc<Node>>,
            late_flags: Option<SymbolFlags>,
            late_decls: Vec<&'a Arc<Node>>,
        }
        let mut groups: std::collections::HashMap<(String, bool), Group> =
            std::collections::HashMap::new();
        let mut order: Vec<(String, bool)> = Vec::new();
        for m in members.iter() {
            if m.kind == SyntaxKind::Constructor {
                continue;
            }
            let Some((name, is_late)) = resolved.get(&m.id()).cloned().flatten() else {
                continue;
            };
            let Some(includes) = member_symbol_includes(m) else {
                continue;
            };
            let key = (name, m.has_syntactic_modifier(ModifierFlags::Static));
            let group = groups.entry(key.clone()).or_insert_with(|| {
                order.push(key.clone());
                Group {
                    early_flags: None,
                    early_decls: Vec::new(),
                    late_flags: None,
                    late_decls: Vec::new(),
                }
            });
            if is_late {
                let conflict = group
                    .late_flags
                    .is_some_and(|l| l.intersects(get_excluded_symbol_flags(includes)));
                if conflict {
                    // Go lateBindMember 冲突：对既有声明与当前声明各报一条
                    let mut all: Vec<&Arc<Node>> = group.early_decls.iter().copied().collect();
                    all.extend(group.late_decls.iter().copied());
                    all.push(m);
                    let display = self
                        .node_source_text(&m.name().unwrap())
                        .unwrap_or_else(|| name.clone());
                    for d in all {
                        let loc = d.name().map(|n| n.loc).unwrap_or(d.loc);
                        self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                            self.current_file.clone(),
                            loc,
                            tsox_core::diagnostics::messages_generated::DUPLICATE_IDENTIFIER_0,
                            vec![display.clone()],
                        ));
                    }
                    if let Some(l) = group.late_flags
                        && l.intersects(SymbolFlags::ACCESSOR)
                        && l & SymbolFlags::ACCESSOR != includes & SymbolFlags::ACCESSOR
                    {
                        group.late_flags = Some(l | SymbolFlags::ACCESSOR);
                    }
                    continue;
                }
                let l = group.late_flags.get_or_insert(SymbolFlags::None);
                *l |= includes;
                group.late_decls.push(m);
            } else {
                let conflict = group
                    .early_flags
                    .is_some_and(|f| f.intersects(get_excluded_symbol_flags(includes)));
                if conflict {
                    // binder 冲突已由 declareSymbol 路径报错；此处仅同步完整
                    // accessor 标记语义
                    if let Some(f) = group.early_flags
                        && f.intersects(SymbolFlags::ACCESSOR)
                        && f & SymbolFlags::ACCESSOR != includes & SymbolFlags::ACCESSOR
                    {
                        group.early_flags = Some(f | SymbolFlags::ACCESSOR);
                    }
                    continue;
                }
                let f = group.early_flags.get_or_insert(SymbolFlags::None);
                *f |= includes;
                group.early_decls.push(m);
            }
        }

        for key in order {
            let Some(group) = groups.get(&key) else {
                continue;
            };
            let (Some(early_flags), Some(late_flags)) = (group.early_flags, group.late_flags)
            else {
                continue;
            };
            if !early_flags.intersects(get_excluded_symbol_flags(late_flags)) {
                continue;
            }
            // Go reportMergeSymbolError：双侧声明各报一条并互挂 related；
            // 显示名取晚绑定符号首个声明的名字原文（如 `[foo]`）
            let display = group
                .late_decls
                .iter()
                .find_map(|d| d.name().and_then(|n| self.node_source_text(&n)))
                .unwrap_or_else(|| key.0.clone());
            let early_nodes: Vec<(tsox_core::core::text::TextRange, Option<Arc<SourceFile>>)> =
                group
                    .early_decls
                    .iter()
                    .map(|d| {
                        (
                            d.name().map(|n| n.loc).unwrap_or(d.loc),
                            self.current_file.clone(),
                        )
                    })
                    .collect();
            let late_nodes: Vec<(tsox_core::core::text::TextRange, Option<Arc<SourceFile>>)> =
                group
                    .late_decls
                    .iter()
                    .map(|d| {
                        (
                            d.name().map(|n| n.loc).unwrap_or(d.loc),
                            self.current_file.clone(),
                        )
                    })
                    .collect();
            for (loc, file) in &early_nodes {
                let related: Vec<_> = late_nodes
                    .iter()
                    .filter(|(rl, _)| rl != loc)
                    .cloned()
                    .collect();
                self.push_dup_error_with_related(
                    file,
                    *loc,
                    tsox_core::diagnostics::messages_generated::DUPLICATE_IDENTIFIER_0,
                    &display,
                    &related,
                );
            }
            for (loc, file) in &late_nodes {
                let related: Vec<_> = early_nodes
                    .iter()
                    .filter(|(rl, _)| rl != loc)
                    .cloned()
                    .collect();
                self.push_dup_error_with_related(
                    file,
                    *loc,
                    tsox_core::diagnostics::messages_generated::DUPLICATE_IDENTIFIER_0,
                    &display,
                    &related,
                );
            }
        }
    }
}
