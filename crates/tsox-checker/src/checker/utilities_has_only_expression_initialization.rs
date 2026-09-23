#![allow(unused_imports)]

use crate::checker::utilities::*;

pub fn has_only_expression_initialization(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::VariableDeclaration
            | SyntaxKind::Parameter
            | SyntaxKind::BindingElement
            | SyntaxKind::PropertyDeclaration
            | SyntaxKind::PropertyAssignment
            | SyntaxKind::EnumMember
    )
}

pub fn is_super_call(n: &Node) -> bool {
    tsox_frontend::ast::is_call_expression(n)
        && n.expression()
            .map(|e| e.kind == SyntaxKind::SuperKeyword)
            .unwrap_or(false)
}

pub fn is_call_chain(node: &Node) -> bool {
    tsox_frontend::ast::is_call_expression(node) && node.flags.contains(NodeFlags::OptionalChain)
}

pub fn is_non_null_access(node: &Node) -> bool {
    tsox_frontend::ast::is_access_expression(node)
        && node
            .expression()
            .map(|e| tsox_frontend::ast::is_non_null_expression(e))
            .unwrap_or(false)
}

pub fn is_this_property(node: &Node) -> bool {
    (tsox_frontend::ast::is_property_access_expression(node)
        || tsox_frontend::ast::is_element_access_expression(node))
        && node
            .expression()
            .map(|e| e.kind == SyntaxKind::ThisKeyword)
            .unwrap_or(false)
}

pub fn is_optional_declaration(_declaration: &Node) -> bool {
    false
}

pub fn is_type_assertion(node: &Node) -> bool {
    tsox_frontend::ast::is_assertion_expression(node)
}

pub fn is_empty_object_literal(expression: &Node) -> bool {
    tsox_frontend::ast::is_object_literal_expression(expression)
}

pub fn is_empty_array_literal(expression: &Node) -> bool {
    tsox_frontend::ast::is_array_literal_expression(expression)
}

pub fn has_type(node: &Node) -> bool {
    node.type_node().is_some()
}

pub fn can_have_flow_node(node: &Node) -> bool {
    let _ = node;
    false
}

pub fn is_private_identifier_symbol(symbol: &Symbol) -> bool {
    symbol.name.starts_with(&format!(
        "{}#",
        tsox_frontend::ast::INTERNAL_SYMBOL_NAME_PREFIX
    ))
}

pub fn is_known_symbol(symbol: &Symbol) -> bool {
    is_late_bound_name(&symbol.name)
}

pub fn is_external_module_symbol(module_symbol: &Symbol) -> bool {
    module_symbol.flags.contains(SymbolFlags::MODULE) && module_symbol.name.starts_with('"')
}

pub fn has_export_assignment_symbol(module_symbol: &Symbol) -> bool {
    module_symbol
        .exports
        .get(tsox_frontend::ast::INTERNAL_SYMBOL_NAME_EXPORT_EQUALS)
        .is_some()
}

pub fn is_static_private_identifier_property(s: &Symbol) -> bool {
    s.value_declaration
        .as_ref()
        .map(|d| tsox_frontend::ast::is_static(d))
        .unwrap_or(false)
}

pub fn get_declarations_of_kind(symbol: &Symbol, kind: SyntaxKind) -> Vec<Arc<Node>> {
    symbol
        .declarations
        .iter()
        .filter(|d| d.kind == kind)
        .cloned()
        .collect()
}

pub fn all_declarations_in_same_source_file(symbol: &Symbol) -> bool {
    if symbol.declarations.len() > 1 {
        let mut source_file_id: Option<u64> = None;
        for (i, d) in symbol.declarations.iter().enumerate() {
            if let Some(sf) = tsox_frontend::ast::get_source_file_of_node(d) {
                if i == 0 {
                    source_file_id = Some(sf.id());
                } else if source_file_id != Some(sf.id()) {
                    return false;
                }
            }
        }
    }
    true
}

pub fn get_index_symbol_from_symbol_table(symbol_table: &SymbolTable) -> Option<Arc<Symbol>> {
    symbol_table
        .get(tsox_frontend::ast::INTERNAL_SYMBOL_NAME_INDEX)
        .cloned()
}

pub fn symbols_to_array(symbols: &SymbolTable) -> Vec<Arc<Symbol>> {
    symbols
        .iter()
        .filter(|(id, _)| !is_reserved_member_name(id))
        .map(|(_, symbol)| Arc::clone(symbol))
        .collect()
}

pub fn create_symbol_table(symbols: &[Arc<Symbol>]) -> SymbolTable {
    let mut result = SymbolTable::new();
    for symbol in symbols {
        result.insert(symbol.name.clone(), Arc::clone(symbol));
    }
    result
}

pub fn is_object_or_array_literal_type(t: &Type) -> bool {
    t.object_flags
        .intersects(ObjectFlags::ObjectLiteral | ObjectFlags::ArrayLiteral)
}

pub fn is_this_type_parameter(t: &Type) -> bool {
    t.flags.contains(TypeFlags::TypeParameter)
        && matches!(&t.data, TypeData::TypeParameter(tp) if tp.is_this_type)
}

pub fn get_type_name_symbol(t: &Type) -> Option<Arc<Symbol>> {
    if let Some(alias) = &t.alias {
        return alias.symbol.clone();
    }
    if t.flags
        .intersects(TypeFlags::TypeParameter | TypeFlags::StringMapping)
        || t.object_flags
            .intersects(OBJECT_FLAGS_CLASS_OR_INTERFACE | ObjectFlags::Reference)
    {
        return t.symbol.clone();
    }
    None
}

pub fn get_object_type_name(t: &Type) -> Option<Arc<Symbol>> {
    if t.object_flags
        .intersects(OBJECT_FLAGS_CLASS_OR_INTERFACE | ObjectFlags::Reference)
    {
        return t.symbol.clone();
    }
    None
}

pub fn get_sort_order_flags(t: &Type) -> u32 {
    if t.flags.intersects(TypeFlags::EnumLiteral | TypeFlags::Enum)
        && !t.flags.contains(TypeFlags::Union)
    {
        return TypeFlags::Enum.bits();
    }
    t.flags.bits()
}

pub fn compare_type_names(t1: &Type, t2: &Type) -> std::cmp::Ordering {
    let s1 = get_type_name_symbol(t1);
    let s2 = get_type_name_symbol(t2);
    if s1.as_ref().map(|s| s.id()) == s2.as_ref().map(|s| s.id()) {
        if let Some(alias) = &t1.alias {
            return compare_type_lists(
                &alias.type_arguments,
                &t2.alias.as_ref().unwrap().type_arguments,
            );
        }
        return std::cmp::Ordering::Equal;
    }
    match (s1, s2) {
        (None, None) => std::cmp::Ordering::Equal,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (Some(_), None) => std::cmp::Ordering::Less,
        (Some(a), Some(b)) => a.name.cmp(&b.name),
    }
}

pub fn compare_type_lists(s1: &[Arc<Type>], s2: &[Arc<Type>]) -> std::cmp::Ordering {
    if s1.len() != s2.len() {
        return s1.len().cmp(&s2.len());
    }
    for (t1, t2) in s1.iter().zip(s2.iter()) {
        let c = compare_types(t1, t2);
        if c != std::cmp::Ordering::Equal {
            return c;
        }
    }
    std::cmp::Ordering::Equal
}

pub fn type_parameters_match(a: &Type, b: &Type) -> bool {
    // 恒等实例（含驻留的延迟 IndexedAccess T[P]）总是匹配
    if std::ptr::eq(a as *const Type, b as *const Type) || a.id == b.id {
        return true;
    }
    // 延迟 IndexedAccess 按成分结构等价（Go 依赖完整驻留，我们补结构判定）
    if let (TypeData::IndexedAccess(x), TypeData::IndexedAccess(y)) = (&a.data, &b.data) {
        let obj_eq = x
            .object_type
            .as_ref()
            .zip(y.object_type.as_ref())
            .is_some_and(|(p, q)| p.id == q.id);
        let idx_eq = x
            .index_type
            .as_ref()
            .zip(y.index_type.as_ref())
            .is_some_and(|(p, q)| p.id == q.id);
        if obj_eq && idx_eq {
            return true;
        }
    }
    if !a.flags.contains(TypeFlags::TypeParameter) || !b.flags.contains(TypeFlags::TypeParameter) {
        return false;
    }
    match (&a.symbol, &b.symbol) {
        (Some(x), Some(y)) => Arc::ptr_eq(x, y),
        _ => false,
    }
}

pub fn compare_union_members(t1: &Type, t2: &Type) -> std::cmp::Ordering {
    get_sort_order_flags(t1)
        .cmp(&get_sort_order_flags(t2))
        .then_with(|| compare_type_names(t1, t2))
}

pub fn compare_types(t1: &Type, t2: &Type) -> std::cmp::Ordering {
    if t1.id == t2.id {
        return std::cmp::Ordering::Equal;
    }
    let c = get_sort_order_flags(t1).cmp(&get_sort_order_flags(t2));
    if c != std::cmp::Ordering::Equal {
        return c;
    }
    let c = compare_type_names(t1, t2);
    if c != std::cmp::Ordering::Equal {
        return c;
    }

    t1.id.cmp(&t2.id)
}

pub fn get_assignment_target_kind(node: &Arc<Node>) -> AssignmentKind {
    let Some(target) = get_assignment_target(node) else {
        return AssignmentKind::None;
    };
    match &target.data {
        tsox_frontend::ast::NodeData::BinaryExpression(bin) => {
            if matches!(
                bin.operator_token.kind,
                SyntaxKind::EqualsToken
                    | SyntaxKind::AmpersandAmpersandEqualsToken
                    | SyntaxKind::BarBarEqualsToken
                    | SyntaxKind::QuestionQuestionEqualsToken
            ) {
                AssignmentKind::Definite
            } else {
                AssignmentKind::Compound
            }
        }
        tsox_frontend::ast::NodeData::PrefixUnaryExpression(_)
        | tsox_frontend::ast::NodeData::PostfixUnaryExpression(_) => AssignmentKind::Compound,
        tsox_frontend::ast::NodeData::ForInOrOfStatement(_) => AssignmentKind::Definite,
        _ => AssignmentKind::None,
    }
}

/// Go isConstTypeReference：`<const>expr` 的断言类型是无实参的
/// `const` 标识符引用（parseIdentifierName 接受关键字作类型名）
pub fn is_const_type_reference(type_node: &Node) -> bool {
    if type_node.kind != tsox_frontend::ast::SyntaxKind::TypeReference {
        return false;
    }
    if let tsox_frontend::ast::NodeData::TypeReferenceNode(d) = &type_node.data {
        if d.type_arguments.is_some() {
            return false;
        }
        if let tsox_frontend::ast::NodeData::Identifier(id) = &d.type_name.data {
            return id.text == "const";
        }
    }
    false
}
