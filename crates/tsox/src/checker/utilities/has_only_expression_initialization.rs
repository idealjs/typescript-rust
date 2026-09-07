#![allow(unused_imports)]

use super::*;

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
    crate::ast::is_call_expression(n)
        && n.expression()
            .map(|e| e.kind == SyntaxKind::SuperKeyword)
            .unwrap_or(false)
}

pub fn is_call_chain(node: &Node) -> bool {
    crate::ast::is_call_expression(node) && node.flags.contains(NodeFlags::OptionalChain)
}

pub fn is_non_null_access(node: &Node) -> bool {
    crate::ast::is_access_expression(node)
        && node
            .expression()
            .map(|e| crate::ast::is_non_null_expression(e))
            .unwrap_or(false)
}

pub fn is_this_property(node: &Node) -> bool {
    (crate::ast::is_property_access_expression(node)
        || crate::ast::is_element_access_expression(node))
        && node
            .expression()
            .map(|e| e.kind == SyntaxKind::ThisKeyword)
            .unwrap_or(false)
}

pub fn is_optional_declaration(_declaration: &Node) -> bool {
    false
}

pub fn is_type_assertion(node: &Node) -> bool {
    crate::ast::is_assertion_expression(node)
}

pub fn is_empty_object_literal(expression: &Node) -> bool {
    crate::ast::is_object_literal_expression(expression)
}

pub fn is_empty_array_literal(expression: &Node) -> bool {
    crate::ast::is_array_literal_expression(expression)
}

pub fn has_type(node: &Node) -> bool {
    node.type_node().is_some()
}

pub fn can_have_flow_node(node: &Node) -> bool {
    let _ = node;
    false
}

pub fn is_private_identifier_symbol(symbol: &Symbol) -> bool {
    symbol
        .name
        .starts_with(&format!("{}#", crate::ast::INTERNAL_SYMBOL_NAME_PREFIX))
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
        .get(crate::ast::INTERNAL_SYMBOL_NAME_EXPORT_EQUALS)
        .is_some()
}

pub fn is_static_private_identifier_property(s: &Symbol) -> bool {
    s.value_declaration
        .as_ref()
        .map(|d| crate::ast::is_static(d))
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
            if let Some(sf) = crate::ast::get_source_file_of_node(d) {
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
        .get(crate::ast::INTERNAL_SYMBOL_NAME_INDEX)
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
    if !a.flags.contains(TypeFlags::TypeParameter) || !b.flags.contains(TypeFlags::TypeParameter) {
        return false;
    }
    if std::ptr::eq(a as *const Type, b as *const Type) {
        return true;
    }
    match (&a.symbol, &b.symbol) {
        (Some(x), Some(y)) => Arc::ptr_eq(x, y),
        _ => false,
    }
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

pub fn get_assignment_target_kind(node: &Node) -> AssignmentKind {
    let Some(target) = get_assignment_target(node) else {
        return AssignmentKind::None;
    };
    match &target.data {
        crate::ast::NodeData::BinaryExpression(bin) => {
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
        crate::ast::NodeData::PrefixUnaryExpression(_)
        | crate::ast::NodeData::PostfixUnaryExpression(_) => AssignmentKind::Compound,
        crate::ast::NodeData::ForInOrOfStatement(_) => AssignmentKind::Definite,
        _ => AssignmentKind::None,
    }
}
