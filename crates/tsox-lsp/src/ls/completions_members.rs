use std::sync::Arc;

use tsox_checker::checker::Checker;
use tsox_checker::checker::types::Type;
use tsox_frontend::ast::{ModifierFlags, Node, NodeData, Symbol, SymbolFlags, SyntaxKind};

use super::completions_accessibility::{
    class_named_like, enclosing_class_of, is_property_accessible,
};
use super::completions_members_receiver::{
    base_identifier, deepest_node_ending_at, primitive_interface_of, source_text_of,
};

/// Go completions.go 右点成员补全（getTypeScriptMemberSymbols 首段）：
/// 节点处于 `recv.` / `recv.par|tial` / 限定名段时，返回接收者可访问成员。
/// 命名空间/枚举接收者给 exports，其余给类型的 apparent properties
/// 返回 (成员列表, 是否处于点后语境)。点后语境下无成员即空列表（Go 不回退
/// scope 符号），非点后语境返回 None 交由 scope 路径
pub(super) fn member_symbols_after_dot(
    checker: &mut Checker,
    node: &Arc<Node>,
    position: usize,
) -> MemberDotResult {
    use tsox_frontend::ast::NodeData;
    use tsox_frontend::ast::SyntaxKind;
    let _dbg_result = (|| {
        let _ = (&NodeData::Identifier, &SyntaxKind::Identifier);
    })();

    // `q..`：第二个点右侧是缺名（Go 取缺失名节点类型 error）→ 无可访问属性；
    // 三个点（spread）不适用，走既有 NotDot 回退
    if let Some((text, _)) = source_text_of(checker, node) {
        let mut p = position.min(text.len());
        while p > 0 {
            let c = text[..p].chars().last().unwrap();
            if c.is_alphanumeric() || c == '_' || c == '$' {
                p -= c.len_utf8();
            } else {
                break;
            }
        }
        if p >= 2
            && &text[p - 2..p] == ".."
            && !(p >= 3 && &text[p - 3..p - 2] == ".")
            // `5..`：数字字面量 `5.` + 成员点（Go isDotOfNumericLiteral）；
            // `foo1..` 仍属双点缺名
            && !(p >= 3 && {
                let prefix = &text[..p - 2];
                let digits = prefix.trim_end_matches(|c: char| {
                    c.is_ascii_digit() || c == '_' || c == '.'
                });
                digits.len() < prefix.len()
                    && digits
                        .chars()
                        .last()
                        .is_none_or(|c| !(c.is_alphanumeric() || c == '_' || c == '$'))
            })
        {
            return MemberDotResult::Dot(Vec::new());
        }
    }

    // 向上找到所属的属性访问/限定名，并确认光标在 '.' 之后（不完整语法下
    // 定位节点可能是访问表达式自身或 '.' token，用光标位置判定）
    let mut access = Arc::clone(node);
    let mut found: Option<Arc<Node>> = None;
    for _ in 0..6 {
        match access.kind {
            SyntaxKind::PropertyAccessExpression => {
                let NodeData::PropertyAccessExpression(d) = &access.data else {
                    break;
                };
                if position <= d.expression.end() {
                    break;
                }
                found = Some(Arc::clone(&access));
                break;
            }
            SyntaxKind::QualifiedName => {
                let NodeData::QualifiedName(d) = &access.data else {
                    break;
                };
                if position <= d.left.end() {
                    break;
                }
                found = Some(Arc::clone(&access));
                break;
            }
            SyntaxKind::DotToken | SyntaxKind::QuestionDotToken => {
                let Some(parent) = access.parent() else {
                    break;
                };
                if parent.kind == SyntaxKind::PropertyAccessExpression {
                    found = Some(parent);
                }
                break;
            }
            _ => {
                // EOF 等位置 deepest 节点是 SourceFile：中断 walk 落到点回退
                let Some(parent) = access.parent() else {
                    break;
                };
                access = parent;
            }
        }
    }
    let access = match found {
        Some(a) => a,
        // 不完整语法（typeof x. 尾部）解析不出 PAE/QN：按 Go contextToken 回退，
        // 光标前（跳过部分键名）是 '.' 时，取以该点结尾的最内表达式为接收者
        None => {
            let Some((text, root)) = source_text_of(checker, node) else {
                return MemberDotResult::NotDot;
            };
            let mut p = position.min(text.len());
            while p > 0 {
                let Some(c) = text[..p].chars().last() else {
                    break;
                };
                // `#` 私有名片段也属被编辑的键名（this.#|）
                if c.is_alphanumeric() || c == '_' || c == '$' || c == '#' {
                    p -= c.len_utf8();
                } else {
                    break;
                }
            }
            let mut q = p;
            while q > 0 && text[..q].chars().last().is_some_and(|c| c.is_whitespace()) {
                q -= text[..q].chars().last().unwrap().len_utf8();
            }
            if q == 0 || &text[q - 1..q] != "." {
                return MemberDotResult::NotDot;
            }
            // spread（`...`）与 `..` 不是成员访问点
            if q >= 2 && &text[q - 2..q - 1] == "." {
                return MemberDotResult::NotDot;
            }
            let dot = q - 1;
            let Some(recv) = deepest_node_ending_at(&root, dot) else {
                // import("./m"). 尾部：成员点被 ImportType 节点的 trailing
                // 区域吞掉（end 越过点），无 end==dot 的节点；marker 处最深
                // 节点即 ImportType 自身
                if node.kind == SyntaxKind::ImportType {
                    return MemberDotResult::Dot(
                        member_symbols_of_receiver(checker, node, false, false)
                            .unwrap_or_default(),
                    );
                }
                return MemberDotResult::Dot(Vec::new());
            };
            return MemberDotResult::Dot(
                member_symbols_of_receiver(
                    checker,
                    &recv,
                    false,
                    receiver_in_namespace_declaration(&recv),
                )
                .unwrap_or_default(),
            );
        }
    };

    let (receiver, is_expression) = match &access.data {
        NodeData::PropertyAccessExpression(d) => (Arc::clone(&d.expression), true),
        NodeData::QualifiedName(d) => {
            // typeof X. 的右段是值位（typeof 取值侧），类型成员不参与
            let in_typeof = {
                let mut cur = access.parent();
                let mut hit = false;
                while let Some(c) = cur {
                    if c.kind == SyntaxKind::TypeQuery {
                        hit = true;
                        break;
                    }
                    if matches!(c.kind, SyntaxKind::SourceFile) {
                        break;
                    }
                    cur = c.parent();
                }
                hit
            };
            (Arc::clone(&d.left), in_typeof)
        }
        _ => return MemberDotResult::NotDot,
    };

    // Go isNamespaceName：namespace N.M 声明名的限定段位
    let is_namespace_name = matches!(access.kind, SyntaxKind::QualifiedName)
        && matches!(access.parent().map(|p| p.kind), Some(SyntaxKind::ModuleDeclaration));

    MemberDotResult::Dot(
        member_symbols_of_receiver(checker, &receiver, is_expression, is_namespace_name)
            .unwrap_or_default(),
    )
}

pub(super) enum MemberDotResult {
    Dot(Vec<Arc<Symbol>>),
    NotDot,
}

fn member_symbols_of_receiver(
    checker: &mut Checker,
    receiver: &Arc<Node>,
    value_only: bool,
    is_namespace_name: bool,
) -> Option<Vec<Arc<Symbol>>> {
    // ImportType 语境（Go getTypeScriptMemberSymbols 的 isImportType 分支）：
    // import("./m"). / import("./m").Q. 点左实体为 ImportType 或其 qualifier 段
    if receiver.kind == SyntaxKind::ImportType || in_import_type_qualifier(receiver) {
        if let Some((base, is_type_of)) = checker.resolve_import_type_member_base(receiver) {
            if base
                .flags
                .intersects(SymbolFlags::ValueModule | SymbolFlags::NamespaceModule | SymbolFlags::ENUM)
            {
                return Some(import_type_module_members(checker, &base, is_type_of));
            }
        }
        // export= 类等非模块基点：receiver 是 ImportType 时落回下方类型通道
        //（get_type_of_node → get_type_from_import_type_node 给值/引用类型）
    }

    // 命名空间/枚举导出：Go GetExportsOfModule + 值/类型位过滤；
    // 限定链（c.Inner.）逐段解析到链尾符号
    if let Some(target) = resolve_module_qualified(checker, receiver) {
        if target
            .flags
            .intersects(SymbolFlags::ValueModule | SymbolFlags::NamespaceModule | SymbolFlags::ENUM)
        {
            // Go forEachExportAndPropertyOfModule：exports ∪ export= 值的属性
            // （JS CommonJS 的 module.exports.* 成员不在文件符号 exports 表）
            let mut exports: Vec<Arc<Symbol>> = Vec::new();
            checker.for_each_export_and_property_of_module(&target, &mut |sym, _| {
                if !exports.iter().any(|e| Arc::ptr_eq(e, sym)) {
                    exports.push(Arc::clone(sym));
                }
            });
            // 枚举成员在 members 表（binder 未入 exports）；Go GetExportsOfModule
            // 对枚举给成员并集
            if target.flags.intersects(SymbolFlags::ENUM) {
                for (k, v) in target.members.entries.iter() {
                    if !exports.iter().any(|e| e.name == *k) {
                        exports.push(Arc::clone(v));
                    }
                }
            }
            // Go isNamespaceName 分支：namespace N.M/**/ 的名位只给 namespace
            // 标志成员，且排除「仅由正在编辑的声明引入」的名字
            let editing_parent = if is_namespace_name {
                receiver.parent().map(|p| p.id())
            } else {
                None
            };
            let mut symbols: Vec<Arc<Symbol>> = Vec::new();
            for e in exports {
                if e.name.is_empty() || e.name.starts_with('\u{FE}') {
                    continue;
                }
                let accessible = if let Some(parent_id) = editing_parent {
                    e.flags.intersects(SymbolFlags::NAMESPACE)
                        && !e
                            .declarations
                            .iter()
                            .all(|d| d.parent().is_some_and(|p| p.id() == parent_id))
                } else if value_only {
                    e.flags.intersects(SymbolFlags::VALUE | SymbolFlags::Assignment)
                        || (e.flags.intersects(SymbolFlags::Alias)
                            && checker
                                .follow_alias_resolving(&e)
                                .is_some_and(|t| {
                                    t.flags.intersects(SymbolFlags::VALUE | SymbolFlags::Assignment)
                                }))
                } else {
                    // 类型位的 X.：仅类型意义（Go
                    // symbolCanBeReferencedAtTypeLocation：Type 联合，或
                    // namespace 的 exports 递归存在可类型引用成员）
                    e.flags.intersects(SymbolFlags::TYPE)
                        || symbol_referenced_at_type_location(checker, &e, &mut Vec::new())
                };
                if accessible {
                    symbols.push(e);
                }
            }
            return Some(symbols);
        }
    }

    // 常规成员：接收者类型的 apparent properties；原始型/字面量经全局接口
    // 解包（Go getApparentType → lib String/Number/...）
    let t = receiver_type(checker, receiver);
    // Go getApparentType：可实例化类型参数取基约束（`x: U extends A` 的 U）
    let t = if t.is_type_parameter() {
        checker
            .get_constraint_of_type_parameter(&t)
            .unwrap_or(t)
    } else {
        t
    };
    // Go addTypeProperties：可空接收者剥掉 null/undefined（`?.` 自动修正，
    // 默认包含该偏好）
    let t = if tsox_checker::checker::utilities::is_nullable_type(&t)
        || (t.is_union()
            && t.types().is_some_and(|ms| {
                ms.iter().any(|m| {
                    m.flags.intersects(
                        tsox_checker::checker::types::TypeFlags::Undefined
                            | tsox_checker::checker::types::TypeFlags::Null,
                    )
                })
            }))
    {
        checker.get_non_nullable_type_of(&t)
    } else {
        t
    };
    let iface = primitive_interface_of(&t);
    if let Some(name) = iface {
        if let Some(props) = checker.global_interface_properties(name) {
            if !props.is_empty() {
                return Some(
                    props
                        .into_iter()
                        .filter(|p| is_property_accessible(receiver, &t, p))
                        .collect(),
                );
            }
        }
        return None;
    }
    let apparent = checker.get_apparent_type(&t);
    let props = checker.get_apparent_properties(&apparent);
    let props: Vec<Arc<Symbol>> = props
        .into_iter()
        .filter(|p| is_property_accessible(receiver, &t, p))
        .collect();
    if props.is_empty() {
        return None;
    }
    Some(props)
}

/// `this` / `super` 接收者的类型：this 走 checker 结构化解析
/// （checkThisExpression/getThisContainer，Go getTypeAtLocation 同源）
fn receiver_type(checker: &mut Checker, receiver: &Arc<Node>) -> Arc<Type> {
    match receiver.kind {
        SyntaxKind::SuperKeyword => base_class_of(receiver)
            .map(|base| checker.build_class_instance_type_with_base(&base))
            .unwrap_or_else(|| checker.get_type_of_node(receiver)),
        // JSX children 回调的无注解形参：类型来自 children 上下文
        //（Go getContextualTypeForChildJsxExpression）
        SyntaxKind::Identifier => {
            super::completions_jsx_children::jsx_children_param_type(checker, receiver)
                .unwrap_or_else(|| checker.get_type_of_node(receiver))
        }
        _ => checker.get_type_of_node(receiver),
    }
}

fn base_class_of(node: &Arc<Node>) -> Option<Arc<Node>> {
    let class = enclosing_class_of(node)?;
    let heritage = match &class.data {
        NodeData::ClassDeclaration(d) => d.heritage_clauses.clone(),
        NodeData::ClassExpression(d) => d.heritage_clauses.clone(),
        _ => None,
    }?;
    for clause in heritage.iter() {
        if clause.kind != SyntaxKind::HeritageClause {
            continue;
        }
        let types = match &clause.data {
            NodeData::HeritageClause(d) => d.types.clone(),
            _ => continue,
        };
        for entry in types.iter() {
            let expr = match &entry.data {
                NodeData::ExpressionWithTypeArguments(d) => Arc::clone(&d.expression),
                _ => Arc::clone(entry),
            };
            let name = match &expr.data {
                NodeData::Identifier(d) => d.text.clone(),
                _ => continue,
            };
            if let Some(base) = class_named_like(node, &name) {
                return Some(base);
            }
        }
    }
    None
}

/// receiver（限定段/标识符）向上是否归属 namespace 声明名（Go
/// isNamespaceName）
fn receiver_in_namespace_declaration(receiver: &Arc<Node>) -> bool {
    let mut cur = receiver.parent();
    while let Some(c) = cur {
        if matches!(c.kind, SyntaxKind::QualifiedName | SyntaxKind::Identifier) {
            cur = c.parent();
            continue;
        }
        return c.kind == SyntaxKind::ModuleDeclaration;
    }
    false
}

/// receiver 是否为某 ImportTypeNode 的 qualifier 链上的标识符/限定名
fn in_import_type_qualifier(receiver: &Arc<Node>) -> bool {
    if !matches!(receiver.kind, SyntaxKind::Identifier | SyntaxKind::QualifiedName) {
        return false;
    }
    let Some(mut cur) = receiver.parent() else {
        return false;
    };
    while cur.kind == SyntaxKind::QualifiedName {
        let Some(parent) = cur.parent() else {
            return false;
        };
        cur = parent;
    }
    if let NodeData::ImportTypeNode(d) = &cur.data {
        return d
            .qualifier
            .as_ref()
            .is_some_and(|q| q.end() >= receiver.end() && q.pos() <= receiver.pos());
    }
    false
}

/// ImportType 语境的模块成员枚举（Go GetExportsOfModule + 值/类型位过滤，
/// 类型位含 namespace 的 exports 递归可引用判定）
fn import_type_module_members(
    checker: &mut Checker,
    module: &Arc<Symbol>,
    value_only: bool,
) -> Vec<Arc<Symbol>> {
    let mut exports: Vec<Arc<Symbol>> = Vec::new();
    checker.for_each_export_and_property_of_module(module, &mut |sym, _| {
        if !exports.iter().any(|e| Arc::ptr_eq(e, sym)) {
            exports.push(Arc::clone(sym));
        }
    });
    let mut out = Vec::new();
    for e in exports {
        if e.name.is_empty() || e.name.starts_with('\u{FE}') {
            continue;
        }
        let accessible = if value_only {
            e.flags.intersects(SymbolFlags::VALUE | SymbolFlags::Assignment)
                || (e.flags.intersects(SymbolFlags::Alias)
                    && checker
                        .follow_alias_resolving(&e)
                        .is_some_and(|t| t.flags.intersects(SymbolFlags::VALUE | SymbolFlags::Assignment)))
        } else {
            let mut seen = Vec::new();
            symbol_referenced_at_type_location(checker, &e, &mut seen)
        };
        if accessible {
            out.push(e);
        }
    }
    out
}

/// Go symbolCanBeReferencedAtTypeLocation：具类型意义，或 namespace 的
/// exports 递归存在可类型引用成员
fn symbol_referenced_at_type_location(
    checker: &mut Checker,
    symbol: &Arc<Symbol>,
    seen: &mut Vec<*const Symbol>,
) -> bool {
    let key = Arc::as_ptr(symbol);
    if seen.contains(&key) {
        return false;
    }
    seen.push(key);
    if symbol.flags.intersects(SymbolFlags::TYPE) {
        return true;
    }
    if symbol.flags.intersects(SymbolFlags::ValueModule | SymbolFlags::NamespaceModule) {
        let mut children = Vec::new();
        checker.for_each_export_and_property_of_module(symbol, &mut |sym, _| {
            if !sym.name.starts_with('\u{FE}') {
                children.push(Arc::clone(sym));
            }
        });
        return children.iter().any(|c| {
            let mut sub_seen = seen.clone();
            symbol_referenced_at_type_location(checker, c, &mut sub_seen)
        });
    }
    false
}

// 限定链接收者（c. / c.Inner. / A.B.C.）解析到链尾符号：基点标识符经
// resolve_identifier + follow_alias_resolving，后续每段在模块/枚举容器的
// exports（含 export * 展开）或 members 里查
fn resolve_module_qualified(
    checker: &mut Checker,
    receiver: &Arc<Node>,
) -> Option<Arc<Symbol>> {
    let mut names: Vec<String> = Vec::new();
    let mut cur = Arc::clone(receiver);
    loop {
        match &cur.data {
            NodeData::Identifier(_) => break,
            NodeData::QualifiedName(d) => {
                names.push(d.right.text().to_string());
                cur = Arc::clone(&d.left);
            }
            NodeData::PropertyAccessExpression(d) => {
                names.push(d.name.text().to_string());
                cur = Arc::clone(&d.expression);
            }
            _ => return None,
        }
    }
    names.reverse();
    let base_sym = checker.resolve_identifier(&cur)?;
    let mut sym = checker
        .follow_alias_resolving(&base_sym)
        .unwrap_or_else(|| base_sym.clone());
    for name in names {
        let resolved = checker.follow_alias_resolving(&sym).unwrap_or_else(|| sym.clone());
        let hit = if resolved.flags.intersects(SymbolFlags::MODULE | SymbolFlags::ENUM) {
            checker
                .try_get_member_in_module_exports(&name, &resolved)
                .or_else(|| resolved.members.get(&name).cloned())
        } else {
            resolved.members.get(&name).cloned()
        };
        sym = hit?;
    }
    Some(sym)
}
