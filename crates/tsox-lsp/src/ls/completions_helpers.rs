use std::sync::Arc;

use crate::lsp::lsproto_lsp::Position;
use tsox_checker::checker::Checker;
use tsox_compile::compiler::Program;
use tsox_frontend::ast::Node;
use tsox_frontend::ast::SourceFile;
use tsox_frontend::ast::Symbol;
use tsox_frontend::ast::SymbolFlags;
use tsox_frontend::ast::SyntaxKind;
use tsox_frontend::ast::node::LineMap;
use tsox_frontend::ast::node_data_generated::for_each_child;

use crate::ls::types::CompletionItem;

pub(super) fn program_build_checker(program: &Arc<Program>) -> Checker {
    program.build_checker()
}

pub(super) fn symbol_to_completion_kind(flags: SymbolFlags) -> u32 {
    const METHOD: u32 = 2;
    const FUNCTION: u32 = 3;
    const CONSTRUCTOR: u32 = 4;
    const FIELD: u32 = 5;
    const VARIABLE: u32 = 6;
    const CLASS: u32 = 7;
    const INTERFACE: u32 = 8;
    const MODULE: u32 = 9;
    pub(crate) const PROPERTY: u32 = 10;
    const ENUM: u32 = 13;
    const KEYWORD: u32 = 14;
    const ENUM_MEMBER: u32 = 20;
    const CONSTANT: u32 = 21;
    const STRUCT: u32 = 22;
    const TYPE_PARAMETER: u32 = 25;

    if flags.contains(SymbolFlags::TypeParameter) {
        return TYPE_PARAMETER;
    }
    if flags.contains(SymbolFlags::Class) {
        return CLASS;
    }
    if flags.contains(SymbolFlags::Interface) {
        return INTERFACE;
    }
    if flags.contains(SymbolFlags::TypeAlias) {
        return STRUCT;
    }
    if flags.contains(SymbolFlags::ENUM) {
        return ENUM;
    }
    if flags.contains(SymbolFlags::EnumMember) {
        return ENUM_MEMBER;
    }
    if flags.contains(SymbolFlags::Function) {
        return FUNCTION;
    }
    if flags.contains(SymbolFlags::Method) {
        return METHOD;
    }
    if flags.contains(SymbolFlags::Constructor) {
        return CONSTRUCTOR;
    }
    if flags.intersects(SymbolFlags::GetAccessor | SymbolFlags::SetAccessor) {
        return PROPERTY;
    }
    if flags.contains(SymbolFlags::Property) {
        return PROPERTY;
    }
    if flags.intersects(SymbolFlags::ValueModule | SymbolFlags::NamespaceModule) {
        return MODULE;
    }
    if flags.contains(SymbolFlags::Alias) {
        return VARIABLE;
    }
    if flags.contains(SymbolFlags::BlockScopedVariable) {
        if flags.contains(SymbolFlags::BlockScopedVariable) {
            return CONSTANT;
        }
        return VARIABLE;
    }
    if flags.contains(SymbolFlags::FunctionScopedVariable) {
        return VARIABLE;
    }
    let _ = KEYWORD;
    let _ = FIELD;
    VARIABLE
}

pub(super) fn symbol_to_completion_item(symbol: &Arc<Symbol>) -> CompletionItem {
    CompletionItem {
        label: symbol.name.clone(),
        kind: Some(symbol_to_completion_kind(symbol.flags)),
        detail: Some(flags_to_detail(&symbol.flags)),
        documentation: None,
        sort_text: None,
        filter_text: None,
        insert_text: Some(symbol.name.clone()),
        insert_text_format: Some(1),
        text_edit: None,
        additional_text_edits: None,
        commit_characters: None,
        data: None,
    }
}

pub(super) fn flags_to_detail(flags: &SymbolFlags) -> String {
    if flags.contains(SymbolFlags::Function) {
        "function".to_string()
    } else if flags.contains(SymbolFlags::Class) {
        "class".to_string()
    } else if flags.contains(SymbolFlags::Interface) {
        "interface".to_string()
    } else if flags.contains(SymbolFlags::TypeAlias) {
        "type".to_string()
    } else if flags.contains(SymbolFlags::ENUM) {
        "enum".to_string()
    } else if flags.contains(SymbolFlags::EnumMember) {
        "enum member".to_string()
    } else if flags.contains(SymbolFlags::Method) {
        "method".to_string()
    } else if flags.contains(SymbolFlags::MODULE) {
        "module".to_string()
    } else if flags.contains(SymbolFlags::VARIABLE) {
        "variable".to_string()
    } else {
        "value".to_string()
    }
}

pub(super) fn collect_scope_symbols_fallback(
    checker: &Checker,
    file: &Arc<SourceFile>,
    _location: &Arc<Node>,
) -> Vec<Arc<Symbol>> {
    let mut result: Vec<Arc<Symbol>> = Vec::new();
    let mut seen: std::collections::HashSet<u64> = std::collections::HashSet::new();

    let symbol_map = checker.program.symbol_map();
    if let Some(locals) = symbol_map.locals_of(&file.node) {
        for sym in locals.entries.values() {
            if seen.insert(sym.id()) {
                result.push(Arc::clone(sym));
            }
        }
    }

    collect_declaration_symbols(checker, &file.node, &mut seen, &mut result);

    result
}

pub(super) fn collect_declaration_symbols(
    checker: &Checker,
    node: &Arc<Node>,
    seen: &mut std::collections::HashSet<u64>,
    result: &mut Vec<Arc<Symbol>>,
) {
    if is_declaration_kind(node.kind) {
        if let Some(sym) = checker.get_symbol_at_location(node) {
            if seen.insert(sym.id()) {
                result.push(sym);
            }
        }
    }

    for_each_child(node, |child| {
        collect_declaration_symbols(checker, child, seen, result);
        false
    });
}

pub(super) fn is_declaration_kind(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::VariableDeclaration
            | SyntaxKind::FunctionDeclaration
            | SyntaxKind::ClassDeclaration
            | SyntaxKind::InterfaceDeclaration
            | SyntaxKind::TypeAliasDeclaration
            | SyntaxKind::EnumDeclaration
            | SyntaxKind::EnumMember
            | SyntaxKind::MethodDeclaration
            | SyntaxKind::MethodSignature
            | SyntaxKind::PropertyDeclaration
            | SyntaxKind::PropertySignature
            | SyntaxKind::GetAccessor
            | SyntaxKind::SetAccessor
            | SyntaxKind::ModuleDeclaration
            | SyntaxKind::Parameter
            | SyntaxKind::ImportSpecifier
            | SyntaxKind::ImportClause
            | SyntaxKind::TypeParameter
    )
}

pub(super) fn find_deepest_node(node: &Arc<Node>, offset: usize) -> Arc<Node> {
    let mut deepest = Arc::clone(node);
    loop {
        let current = Arc::clone(&deepest);
        let mut next: Option<Arc<Node>> = None;
        for_each_child(&current, |child| {
            if child.pos() <= offset && offset < child.end() {
                next = Some(Arc::clone(child));
                true
            } else {
                false
            }
        });
        match next {
            Some(child) => deepest = child,
            None => break,
        }
    }
    deepest
}

pub(super) fn lsp_position_to_offset(line_map: &LineMap, position: &Position) -> usize {
    let line = position.line as usize;
    let character = position.character as usize;
    let line_start = line_map.line_starts.get(line).copied().unwrap_or(0) as usize;
    line_start + character
}

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
                let Some(parent) = access.parent.clone() else {
                    break;
                };
                if parent.kind == SyntaxKind::PropertyAccessExpression {
                    found = Some(parent);
                }
                break;
            }
            _ => {
                // EOF 等位置 deepest 节点是 SourceFile：中断 walk 落到点回退
                let Some(parent) = access.parent.clone() else {
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
                if c.is_alphanumeric() || c == '_' || c == '$' {
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
            let dot = q - 1;
            let Some(recv) = deepest_node_ending_at(&root, dot) else {
                return MemberDotResult::Dot(Vec::new());
            };
            return MemberDotResult::Dot(
                member_symbols_of_receiver(checker, &recv, false).unwrap_or_default(),
            );
        }
    };

    let (receiver, is_expression) = match &access.data {
        NodeData::PropertyAccessExpression(d) => (Arc::clone(&d.expression), true),
        NodeData::QualifiedName(d) => {
            // typeof X. 的右段是值位（typeof 取值侧），类型成员不参与
            let in_typeof = {
                let mut cur = access.parent.clone();
                let mut hit = false;
                while let Some(c) = cur {
                    if c.kind == SyntaxKind::TypeQuery {
                        hit = true;
                        break;
                    }
                    if matches!(c.kind, SyntaxKind::SourceFile) {
                        break;
                    }
                    cur = c.parent.clone();
                }
                hit
            };
            (Arc::clone(&d.left), in_typeof)
        }
        _ => return MemberDotResult::NotDot,
    };

    MemberDotResult::Dot(
        member_symbols_of_receiver(checker, &receiver, is_expression).unwrap_or_default(),
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
) -> Option<Vec<Arc<Symbol>>> {
    // 命名空间/枚举导出：Go GetExportsOfModule + 值/类型位过滤
    if let Some(base) = base_identifier(receiver) {
        if let Some(sym) = checker.resolve_identifier(&base) {
            let target = checker.follow_alias(&sym).unwrap_or(sym);
            if target
                .flags
                .intersects(SymbolFlags::ValueModule | SymbolFlags::NamespaceModule | SymbolFlags::ENUM)
            {
                let mut exports = checker.get_exports_of_module(&target);
                // 枚举成员在 members 表（binder 未入 exports）；Go GetExportsOfModule
                // 对枚举给成员并集
                if target.flags.intersects(SymbolFlags::ENUM) {
                    for (k, v) in target.members.entries.iter() {
                        if !exports.iter().any(|e| e.name == *k) {
                            exports.push(Arc::clone(v));
                        }
                    }
                }
                let symbols = exports
                    .into_iter()
                    .filter(|e| {
                        !e.name.is_empty()
                            && !e.name.starts_with('\u{FE}')
                            && if value_only {
                                // typeof X.：值位（Go isValidValueAccess）
                                e.flags.intersects(SymbolFlags::VALUE)
                            } else {
                                // 类型位的 X.：仅类型意义（Go
                                // symbolCanBeReferencedAtTypeLocation：namespace
                                // 与纯值导出不参与）
                                e.flags.intersects(SymbolFlags::TYPE)
                            }
                    })
                    .collect();
                return Some(symbols);
            }
        }
    }

    // 常规成员：接收者类型的 apparent properties；原始型/字面量经全局接口
    // 解包（Go getApparentType → lib String/Number/...）
    let t = checker.get_type_of_node(receiver);
    let iface = primitive_interface_of(&t);
    if let Some(name) = iface {
        if let Some(props) = checker.global_interface_properties(name) {
            if !props.is_empty() {
                return Some(props);
            }
        }
        return None;
    }
    let t = checker.get_apparent_type(&t);
    let props = checker.get_apparent_properties(&t);
    if props.is_empty() {
        return None;
    }
    Some(props)
}

fn primitive_interface_of(t: &tsox_checker::checker::types::Type) -> Option<&'static str> {
    use tsox_checker::checker::types::TypeFlags;
    if t.flags.intersects(TypeFlags::String | TypeFlags::StringLiteral) {
        Some("String")
    } else if t.flags.intersects(TypeFlags::Number | TypeFlags::NumberLiteral) {
        Some("Number")
    } else if t.flags.intersects(TypeFlags::Boolean | TypeFlags::BooleanLiteral) {
        Some("Boolean")
    } else if t.flags.intersects(TypeFlags::ESSymbol | TypeFlags::UniqueESSymbol) {
        Some("Symbol")
    } else {
        None
    }
}

fn source_text_of(checker: &Checker, node: &Arc<Node>) -> Option<(String, Arc<Node>)> {
    let sf = checker.get_source_file_of_node(node)?;
    let mut cur = Arc::clone(node);
    while cur.kind != tsox_frontend::ast::SyntaxKind::SourceFile {
        cur = cur.parent.clone()?;
    }
    Some((sf.text.clone(), cur))
}

fn deepest_node_ending_at(root: &Arc<Node>, end: usize) -> Option<Arc<Node>> {
    use tsox_frontend::ast::node_data_generated::for_each_child;
    // 沿包含 end 的孩子下降（尾随 '.' 会被外层语句吞掉），途经 end==dot 的
    // 最深节点即接收者表达式
    let mut best: Option<Arc<Node>> = None;
    if root.pos() < end && root.end() == end {
        best = Some(Arc::clone(root));
    }
    let mut children = Vec::new();
    for_each_child(root, |c| {
        children.push(Arc::clone(c));
        false
    });
    for c in children {
        if c.pos() < end && c.end() == end {
            // 最浅命中即接收者（继续下降会落进尾部实参列表/子表达式）
            return Some(c);
        }
        if c.pos() <= end && c.end() > end {
            if let Some(deeper) = deepest_node_ending_at(&c, end) {
                return Some(deeper);
            }
        }
    }
    best
}

fn base_identifier(node: &Arc<Node>) -> Option<Arc<Node>> {
    use tsox_frontend::ast::NodeData;
    match &node.data {
        NodeData::Identifier(_) => Some(Arc::clone(node)),
        NodeData::QualifiedName(d) => base_identifier(&d.left),
        NodeData::PropertyAccessExpression(d) => base_identifier(&d.expression),
        _ => None,
    }
}
