#![allow(dead_code)]

use std::sync::Arc;

use crate::ast::{
    ModifierFlags, Node, NodeFlags, Symbol, SymbolFlags, SymbolTable, SyntaxKind,
    get_combined_modifier_flags,
    is_element_access_expression, is_property_access_expression,
    is_qualified_name,
    is_variable_declaration_list, is_variable_statement,
};

use super::types::*;

pub fn token_is_identifier_or_keyword(kind: SyntaxKind) -> bool {
    (kind as u32) >= (SyntaxKind::Identifier as u32)
}

pub fn token_is_identifier_or_keyword_or_greater_than(kind: SyntaxKind) -> bool {
    kind == SyntaxKind::GreaterThanToken || token_is_identifier_or_keyword(kind)
}

pub fn can_have_locals(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::ArrowFunction
            | SyntaxKind::Block
            | SyntaxKind::CallSignature
            | SyntaxKind::CaseBlock
            | SyntaxKind::CatchClause
            | SyntaxKind::ClassStaticBlockDeclaration
            | SyntaxKind::ConditionalType
            | SyntaxKind::Constructor
            | SyntaxKind::ConstructorType
            | SyntaxKind::ConstructSignature
            | SyntaxKind::ForStatement
            | SyntaxKind::ForInStatement
            | SyntaxKind::ForOfStatement
            | SyntaxKind::FunctionDeclaration
            | SyntaxKind::FunctionExpression
            | SyntaxKind::FunctionType
            | SyntaxKind::GetAccessor
            | SyntaxKind::IndexSignature
            | SyntaxKind::MappedType
            | SyntaxKind::MethodDeclaration
            | SyntaxKind::MethodSignature
            | SyntaxKind::ModuleDeclaration
            | SyntaxKind::SetAccessor
            | SyntaxKind::SourceFile
            | SyntaxKind::TypeAliasDeclaration
    )
}

pub fn is_any_or_unknown_type(t: &Type) -> bool {
    t.flags.intersects(TYPE_FLAGS_ANY_OR_UNKNOWN)
}

pub fn is_nullable_type(t: &Type) -> bool {
    t.flags.intersects(TYPE_FLAGS_NULLABLE)
}

pub fn is_literal_type(t: &Type) -> bool {
    t.flags.intersects(TYPE_FLAGS_LITERAL)
}

pub fn is_unit_type(t: &Type) -> bool {
    t.flags.intersects(TYPE_FLAGS_UNIT)
}

pub fn is_string_like_type(t: &Type) -> bool {
    t.flags.intersects(TYPE_FLAGS_STRING_LIKE)
}

pub fn is_number_like_type(t: &Type) -> bool {
    t.flags.intersects(TYPE_FLAGS_NUMBER_LIKE)
}

pub fn is_boolean_like_type(t: &Type) -> bool {
    t.flags.intersects(TYPE_FLAGS_BOOLEAN_LIKE)
}

pub fn is_enum_like_type(t: &Type) -> bool {
    t.flags.intersects(TYPE_FLAGS_ENUM_LIKE)
}

pub fn is_primitive_type(t: &Type) -> bool {
    t.flags.intersects(TYPE_FLAGS_PRIMITIVE)
}

pub fn is_definitely_falsy_type(t: &Type) -> bool {
    t.flags.intersects(TYPE_FLAGS_DEFINITELY_FALSY)
}

pub fn is_possibly_falsy_type(t: &Type) -> bool {
    t.flags.intersects(TYPE_FLAGS_POSSIBLY_FALSY)
}

pub fn is_definitely_non_nullable_type(t: &Type) -> bool {
    t.flags.intersects(TYPE_FLAGS_DEFINITELY_NON_NULLABLE)
}

pub fn is_structured_type(t: &Type) -> bool {
    t.flags.intersects(TYPE_FLAGS_STRUCTURED_TYPE)
}

pub fn is_instantiable_type(t: &Type) -> bool {
    t.flags.intersects(TYPE_FLAGS_INSTANTIABLE)
}

pub fn is_type_variable(t: &Type) -> bool {
    t.flags.intersects(TYPE_FLAGS_TYPE_VARIABLE)
}

pub fn is_union_or_intersection_type(t: &Type) -> bool {
    t.flags.intersects(TYPE_FLAGS_UNION_OR_INTERSECTION)
}

pub fn is_object_flags_type(t: &Type) -> bool {
    t.flags.intersects(TYPE_FLAGS_OBJECT_FLAGS_TYPE)
}

pub fn is_freshable_type(t: &Type) -> bool {
    t.flags.intersects(TYPE_FLAGS_FRESHABLE)
}

pub fn is_singleton_type(t: &Type) -> bool {
    t.flags.intersects(TYPE_FLAGS_SINGLETON)
}

pub fn is_narrowable_type(t: &Type) -> bool {
    t.flags.intersects(TYPE_FLAGS_NARROWABLE)
}

pub fn is_intrinsic_type(t: &Type) -> bool {
    t.flags.intersects(TYPE_FLAGS_INTRINSIC)
}

pub fn is_void_like_type(t: &Type) -> bool {
    t.flags.intersects(TYPE_FLAGS_VOID_LIKE)
}

pub fn is_class_type(t: &Type) -> bool {
    t.object_flags.contains(ObjectFlags::Class)
}

pub fn is_interface_type(t: &Type) -> bool {
    t.object_flags.contains(ObjectFlags::Interface)
}

pub fn is_class_or_interface_type(t: &Type) -> bool {
    t.object_flags.intersects(OBJECT_FLAGS_CLASS_OR_INTERFACE)
}

pub fn is_type_reference(t: &Type) -> bool {
    t.object_flags.contains(ObjectFlags::Reference)
}

pub fn is_anonymous_type(t: &Type) -> bool {
    t.object_flags.contains(ObjectFlags::Anonymous)
}

pub fn is_mapped_type(t: &Type) -> bool {
    t.object_flags.contains(ObjectFlags::Mapped)
}

pub fn is_tuple_type(t: &Type) -> bool {

    if t.flags.contains(TypeFlags::Object) {
        t.object_flags.contains(ObjectFlags::Tuple)
            || (t.object_flags.contains(ObjectFlags::Reference)
                && t.target()
                    .map(|target| target.object_flags.contains(ObjectFlags::Tuple))
                    .unwrap_or(false))
    } else {
        false
    }
}

pub fn is_evolving_array_type(t: &Type) -> bool {
    t.flags.contains(TypeFlags::Object) && t.object_flags.contains(ObjectFlags::EvolvingArray)
}

pub fn is_fresh_object_literal_type(t: &Type) -> bool {
    t.flags.contains(TypeFlags::Object) && t.object_flags.contains(ObjectFlags::FreshLiteral)
}

pub fn is_object_literal_type(t: &Type) -> bool {
    t.flags.contains(TypeFlags::Object) && t.object_flags.contains(ObjectFlags::ObjectLiteral)
}

pub fn is_type_usable_as_property_name(t: &Type) -> bool {
    t.flags
        .intersects(TYPE_FLAGS_STRING_OR_NUMBER_LITERAL_OR_UNIQUE | TypeFlags::ESSymbol)
}

pub fn get_property_name_from_type(t: &Type) -> String {
    if t.flags.contains(TypeFlags::StringLiteral) {
        if let Some(LiteralValue::String(s)) = t.literal_value() {
            return s.clone();
        }
    }
    if t.flags.contains(TypeFlags::NumberLiteral) {
        if let Some(LiteralValue::Number(n)) = t.literal_value() {
            return n.to_string();
        }
    }
    if t.flags.contains(TypeFlags::UniqueESSymbol) {
        if let TypeData::UniqueESSymbol(u) = &t.data {
            return u.name.clone();
        }
    }
    String::new()
}

pub fn type_to_string(t: &Type) -> String {

    if let Some(name) = t.intrinsic_name() {
        return name.to_string();
    }

    if let Some(val) = t.literal_value() {
        return val.to_string();
    }

    if t.is_union() {
        if let Some(types) = t.types() {
            let parts: Vec<String> = types.iter().map(|ty| type_to_string(ty)).collect();
            return parts.join(" | ");
        }
    }

    if t.is_intersection() {
        if let Some(types) = t.types() {
            let parts: Vec<String> = types.iter().map(|ty| type_to_string(ty)).collect();
            return parts.join(" & ");
        }
    }

    if t.is_type_parameter() {
        if let TypeData::TypeParameter(tp) = &t.data {
            if tp.is_this_type {
                return "this".to_string();
            }
        }
        if let Some(sym) = &t.symbol {
            return sym.name.clone();
        }
        return "T".to_string();
    }

    if let Some(sym) = &t.symbol {
        return sym.name.clone();
    }

    if t.flags.contains(TypeFlags::Never) {
        return "never".to_string();
    }
    if t.flags.contains(TypeFlags::Object) {
        return "object".to_string();
    }

    "<unknown type>".to_string()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AssignmentKind {
    #[default]
    None,
    Definite,
    Compound,
}

impl ExhaustiveState {
    pub fn is_true(self) -> bool {
        self == ExhaustiveState::True
    }

    pub fn is_false(self) -> bool {
        self == ExhaustiveState::False
    }

    pub fn is_unknown(self) -> bool {
        self == ExhaustiveState::Unknown
    }

    pub fn is_computing(self) -> bool {
        self == ExhaustiveState::Computing
    }
}

pub fn is_late_bound_symbol(_symbol: &Symbol) -> bool {

    false
}

pub fn is_value_symbol(symbol: &Symbol) -> bool {
    symbol.flags.intersects(crate::ast::SymbolFlags::VALUE)
}

pub fn is_type_symbol(symbol: &Symbol) -> bool {
    symbol.flags.intersects(crate::ast::SymbolFlags::TYPE)
}

pub fn is_namespace_symbol(symbol: &Symbol) -> bool {
    symbol.flags.intersects(crate::ast::SymbolFlags::NAMESPACE)
}

pub fn is_optional_symbol(symbol: &Symbol) -> bool {
    symbol.flags.intersects(crate::ast::SymbolFlags::Optional)
}

pub fn is_class_member_symbol(symbol: &Symbol) -> bool {
    symbol
        .flags
        .intersects(crate::ast::SymbolFlags::CLASS_MEMBER)
}

pub fn is_type_any(t: &Type) -> bool {
    t.flags.contains(TypeFlags::Any)
}

pub fn is_type_unknown(t: &Type) -> bool {
    t.flags.contains(TypeFlags::Unknown)
}

pub fn is_type_never(t: &Type) -> bool {
    t.flags.contains(TypeFlags::Never)
}

pub fn is_type_void(t: &Type) -> bool {
    t.flags.contains(TypeFlags::Void)
}

pub fn is_type_undefined(t: &Type) -> bool {
    t.flags.contains(TypeFlags::Undefined)
}

pub fn is_type_null(t: &Type) -> bool {
    t.flags.contains(TypeFlags::Null)
}

pub fn is_type_string(t: &Type) -> bool {
    t.flags.contains(TypeFlags::String)
}

pub fn is_type_number(t: &Type) -> bool {
    t.flags.contains(TypeFlags::Number)
}

pub fn is_type_boolean(t: &Type) -> bool {
    t.flags.contains(TypeFlags::Boolean)
}

pub fn is_type_bigint(t: &Type) -> bool {
    t.flags.contains(TypeFlags::BigInt)
}

pub fn is_symbol_type(t: &Type) -> bool {
    t.flags.contains(TypeFlags::ESSymbol)
}

pub fn is_type_non_primitive(t: &Type) -> bool {
    t.flags.contains(TypeFlags::NonPrimitive)
}

pub fn is_type_error(t: &Type) -> bool {
    t.intrinsic_name() == Some("error")
}

pub fn is_fresh_literal_type(t: &Type) -> bool {
    if let TypeData::Literal(lit) = &t.data {
        lit.regular_type.get().is_some()
    } else {
        false
    }
}

pub fn is_array_type(t: &Type) -> bool {
    t.flags.contains(TypeFlags::Object)
        && t.object_flags.contains(ObjectFlags::Reference)
        && t.target()
            .map(|target| {
                target.object_flags.contains(ObjectFlags::Reference)
                    && target
                        .intrinsic_name()
                        .map(|name| name == "Array")
                        .unwrap_or(false)
            })
            .unwrap_or(false)
}

pub fn is_array_or_tuple_type(t: &Type) -> bool {
    is_array_type(t) || is_tuple_type(t)
}

pub fn is_computed_property_name(name: &str) -> bool {
    name.starts_with('[')
}

pub fn is_internal_symbol_name(name: &str) -> bool {
    name.starts_with(crate::ast::INTERNAL_SYMBOL_NAME_PREFIX)
}

pub fn is_numeric_literal_name(name: &str) -> bool {
    name.parse::<f64>().is_ok()
}

pub fn get_numeric_literal_name(name: &str) -> String {
    if let Ok(n) = name.parse::<f64>() {
        crate::jsnum::Number(n).to_string()
    } else {
        name.to_string()
    }
}

pub fn is_exponentiation_operator(kind: SyntaxKind) -> bool {
    kind == SyntaxKind::AsteriskAsteriskToken
}

pub fn is_multiplicative_operator(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::AsteriskToken | SyntaxKind::SlashToken | SyntaxKind::PercentToken
    )
}

pub fn is_multiplicative_operator_or_higher(kind: SyntaxKind) -> bool {
    is_exponentiation_operator(kind) || is_multiplicative_operator(kind)
}

pub fn is_additive_operator(kind: SyntaxKind) -> bool {
    matches!(kind, SyntaxKind::PlusToken | SyntaxKind::MinusToken)
}

pub fn is_additive_operator_or_higher(kind: SyntaxKind) -> bool {
    is_additive_operator(kind) || is_multiplicative_operator_or_higher(kind)
}

pub fn is_shift_operator(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::LessThanLessThanToken
            | SyntaxKind::GreaterThanGreaterThanToken
            | SyntaxKind::GreaterThanGreaterThanGreaterThanToken
    )
}

pub fn is_shift_operator_or_higher(kind: SyntaxKind) -> bool {
    is_shift_operator(kind) || is_additive_operator_or_higher(kind)
}

pub fn is_relational_operator(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::LessThanToken
            | SyntaxKind::LessThanEqualsToken
            | SyntaxKind::GreaterThanToken
            | SyntaxKind::GreaterThanEqualsToken
            | SyntaxKind::InstanceOfKeyword
            | SyntaxKind::InKeyword
    )
}

pub fn is_relational_operator_or_higher(kind: SyntaxKind) -> bool {
    is_relational_operator(kind) || is_shift_operator_or_higher(kind)
}

pub fn is_equality_operator(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::EqualsEqualsToken
            | SyntaxKind::EqualsEqualsEqualsToken
            | SyntaxKind::ExclamationEqualsToken
            | SyntaxKind::ExclamationEqualsEqualsToken
    )
}

pub fn is_equality_operator_or_higher(kind: SyntaxKind) -> bool {
    is_equality_operator(kind) || is_relational_operator_or_higher(kind)
}

pub fn is_bitwise_operator(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::AmpersandToken | SyntaxKind::BarToken | SyntaxKind::CaretToken
    )
}

pub fn is_bitwise_operator_or_higher(kind: SyntaxKind) -> bool {
    is_bitwise_operator(kind) || is_equality_operator_or_higher(kind)
}

pub fn is_logical_operator_or_higher(kind: SyntaxKind) -> bool {
    crate::ast::is_logical_binary_operator(kind) || is_bitwise_operator_or_higher(kind)
}

pub fn is_assignment_operator_or_higher(kind: SyntaxKind) -> bool {
    kind == SyntaxKind::QuestionQuestionToken
        || is_logical_operator_or_higher(kind)
        || crate::ast::is_assignment_operator(kind)
}

pub fn is_binary_operator(kind: SyntaxKind) -> bool {
    is_assignment_operator_or_higher(kind) || kind == SyntaxKind::CommaToken
}

pub fn has_override_modifier(node: &Node) -> bool {
    crate::ast::has_syntactic_modifier(node, ModifierFlags::Override)
}

pub fn has_async_modifier(node: &Node) -> bool {
    crate::ast::has_syntactic_modifier(node, ModifierFlags::Async)
}

pub fn get_selected_modifier_flags(node: &Node, flags: ModifierFlags) -> ModifierFlags {
    node.syntactic_modifier_flags() & flags
}

pub fn has_readonly_modifier(node: &Node) -> bool {
    crate::ast::has_syntactic_modifier(node, ModifierFlags::Readonly)
}

pub fn is_infinity_or_nan_string(name: &str) -> bool {
    name == "Infinity" || name == "-Infinity" || name == "NaN"
}

pub fn is_reserved_member_name(name: &str) -> bool {
    let bytes = name.as_bytes();
    bytes.len() >= 2 && bytes[0] == 0xFE && bytes[1] != b'@' && bytes[1] != b'#'
}

pub fn is_late_bound_name(name: &str) -> bool {
    let bytes = name.as_bytes();
    bytes.len() >= 2 && bytes[0] == 0xFE && bytes[1] == b'@'
}

pub fn is_exclamation_token(node: &Node) -> bool {
    node.kind == SyntaxKind::ExclamationToken
}

pub fn is_type_alias(node: &Node) -> bool {
    matches!(
        node.kind,
        SyntaxKind::TypeAliasDeclaration | SyntaxKind::JSTypeAliasDeclaration
    )
}

pub fn is_literal_expression_of_object(node: &Node) -> bool {
    matches!(
        node.kind,
        SyntaxKind::ObjectLiteralExpression
            | SyntaxKind::ArrayLiteralExpression
            | SyntaxKind::RegularExpressionLiteral
            | SyntaxKind::FunctionExpression
            | SyntaxKind::ClassExpression
    )
}

pub fn introduces_arguments_exotic_object(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::MethodDeclaration
            | SyntaxKind::MethodSignature
            | SyntaxKind::Constructor
            | SyntaxKind::GetAccessor
            | SyntaxKind::SetAccessor
            | SyntaxKind::FunctionDeclaration
            | SyntaxKind::FunctionExpression
    )
}

pub fn node_starts_new_lexical_environment(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::Constructor
            | SyntaxKind::FunctionExpression
            | SyntaxKind::FunctionDeclaration
            | SyntaxKind::ArrowFunction
            | SyntaxKind::MethodDeclaration
            | SyntaxKind::GetAccessor
            | SyntaxKind::SetAccessor
            | SyntaxKind::ModuleDeclaration
            | SyntaxKind::SourceFile
    )
}

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
    if !a.flags.contains(TypeFlags::TypeParameter)
        || !b.flags.contains(TypeFlags::TypeParameter)
    {
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

fn get_assignment_target(node: &Node) -> Option<&Node> {
    fn is_assignment_operator(kind: SyntaxKind) -> bool {
        use SyntaxKind::*;
        matches!(
            kind,
            EqualsToken
                | PlusEqualsToken
                | MinusEqualsToken
                | AsteriskEqualsToken
                | SlashEqualsToken
                | PercentEqualsToken
                | AsteriskAsteriskEqualsToken
                | LessThanLessThanEqualsToken
                | GreaterThanGreaterThanEqualsToken
                | GreaterThanGreaterThanGreaterThanEqualsToken
                | AmpersandEqualsToken
                | BarEqualsToken
                | CaretEqualsToken
                | AmpersandAmpersandEqualsToken
                | BarBarEqualsToken
                | QuestionQuestionEqualsToken
        )
    }
    let mut current: &Node = node;
    loop {
        let parent = current.parent.as_ref()?;
        match &parent.data {
            crate::ast::NodeData::BinaryExpression(bin) => {
                let on_path = Arc::as_ref(&bin.left) as *const Node == current;
                return if on_path && is_assignment_operator(bin.operator_token.kind) {
                    Some(parent)
                } else {
                    None
                };
            }
            crate::ast::NodeData::PrefixUnaryExpression(pre) => {
                let incdec = matches!(
                    pre.operator,
                    SyntaxKind::PlusPlusToken | SyntaxKind::MinusMinusToken
                );
                let on_path = Arc::as_ref(&pre.operand) as *const Node == current;
                return if incdec && on_path {
                    Some(parent)
                } else {
                    None
                };
            }
            crate::ast::NodeData::PostfixUnaryExpression(post) => {
                let incdec = matches!(
                    post.operator,
                    SyntaxKind::PlusPlusToken | SyntaxKind::MinusMinusToken
                );
                let on_path = Arc::as_ref(&post.operand) as *const Node == current;
                return if incdec && on_path {
                    Some(parent)
                } else {
                    None
                };
            }
            crate::ast::NodeData::ForInOrOfStatement(for_stmt) => {
                let on_path = Arc::as_ref(&for_stmt.initializer) as *const Node == current;
                return if on_path { Some(parent) } else { None };
            }
            crate::ast::NodeData::ParenthesizedExpression(_) => {
                current = parent;
            }
            _ => return None,
        }
    }
}

pub fn is_compound_like_assignment(assignment: &Node) -> bool {

    let crate::ast::NodeData::BinaryExpression(bin) = &assignment.data else {
        return false;
    };
    if bin.operator_token.kind != SyntaxKind::EqualsToken {
        return false;
    }
    let mut right = &bin.right;
    while let crate::ast::NodeData::ParenthesizedExpression(p) = &right.data {
        right = &p.expression;
    }
    matches!(&right.data, crate::ast::NodeData::BinaryExpression(rhs)
        if is_shift_operator_or_higher(rhs.operator_token.kind))
}

pub fn is_in_compound_like_assignment(node: &Node) -> bool {
    let Some(target) = get_assignment_target(node) else {
        return false;
    };

    let crate::ast::NodeData::BinaryExpression(bin) = &target.data else {
        return false;
    };
    bin.operator_token.kind == SyntaxKind::EqualsToken && is_compound_like_assignment(target)
}

pub fn is_delete_target(node: &Node) -> bool {

    if !crate::ast::is_access_expression(node) {
        return false;
    }
    node.parent
        .as_ref()
        .map(|p| p.kind == SyntaxKind::DeleteExpression)
        .unwrap_or(false)
}

pub fn is_right_side_of_access_expression(node: &Node) -> bool {
    if let Some(parent) = &node.parent {
        if is_property_access_expression(parent) {
            return parent
                .name()
                .map(|n| std::ptr::eq(n.as_ref(), node))
                .unwrap_or(false);
        }
        if is_element_access_expression(parent) {
            return parent
                .expression()
                .map(|e| std::ptr::eq(e.as_ref(), node))
                .unwrap_or(false);
        }
    }
    false
}

pub fn is_top_level_in_external_module_augmentation(node: &Node) -> bool {

    let _ = node;
    false
}

pub fn is_syntactic_default(node: &Node) -> bool {

    matches!(
        node.kind,
        SyntaxKind::ExportSpecifier | SyntaxKind::NamespaceExportDeclaration
    ) || node.has_syntactic_modifier(ModifierFlags::Default)
}

pub fn is_type_reference_identifier(node: &Node) -> bool {

    node.parent
        .as_ref()
        .map(|p| crate::ast::is_type_reference_node(p))
        .unwrap_or(false)
}

pub fn is_in_type_query(node: &Node) -> bool {

    let _ = node;
    false
}

pub fn is_side_effect_import(node: &Node) -> bool {

    let _ = node;
    false
}

pub fn get_external_module_require_argument(node: &Node) -> Option<Arc<Node>> {

    let _ = node;
    None
}

pub fn is_shorthand_ambient_module(node: &Node) -> bool {
    node.kind == SyntaxKind::ModuleDeclaration

}

pub fn is_shorthand_ambient_module_symbol(module_symbol: &Symbol) -> bool {
    module_symbol
        .value_declaration
        .as_ref()
        .map(|d| is_shorthand_ambient_module(d))
        .unwrap_or(false)
}

pub fn entity_name_to_string(name: &Node) -> String {

    name.text().to_string()
}

pub fn get_containing_qualified_name_node(node: &Arc<Node>) -> Arc<Node> {
    let mut result = Arc::clone(node);
    let mut current = node.parent.clone();
    while let Some(ref parent) = current {
        if is_qualified_name(parent) {
            result = Arc::clone(parent);
            current = parent.parent.clone();
        } else {
            break;
        }
    }
    result
}

pub fn is_const_type_reference(node: &Node) -> bool {

    crate::ast::is_type_reference_node(node) && node.text() == "const"
}

pub fn get_single_variable_of_variable_statement(node: &Node) -> Option<Arc<Node>> {

    let _ = node;
    None
}

pub fn is_jsx_intrinsic_tag_name(tag_name: &Node) -> bool {
    crate::ast::is_identifier(tag_name) || crate::ast::is_jsx_namespaced_name(tag_name)
}

pub fn walk_up_outer_expressions(node: &Node) -> Option<Arc<Node>> {

    node.parent.clone()
}

pub fn get_containing_function_or_class_static_block(node: &Node) -> Option<Arc<Node>> {
    node.parent.as_ref().and_then(|parent| {
        crate::ast::find_ancestor(parent, |n| {
            crate::ast::is_function_like_or_class_static_block_declaration(n)
        })
    })
}

pub fn get_enclosing_container(node: &Node) -> Option<Arc<Node>> {

    node.parent.as_ref().and_then(|parent| {
        crate::ast::find_ancestor(parent, |n| {
            matches!(
                n.kind,
                SyntaxKind::FunctionDeclaration
                    | SyntaxKind::FunctionExpression
                    | SyntaxKind::ArrowFunction
                    | SyntaxKind::MethodDeclaration
                    | SyntaxKind::GetAccessor
                    | SyntaxKind::SetAccessor
                    | SyntaxKind::Constructor
                    | SyntaxKind::ClassDeclaration
                    | SyntaxKind::ClassExpression
                    | SyntaxKind::ModuleDeclaration
                    | SyntaxKind::SourceFile
            )
        })
    })
}

pub fn is_this_initialized_declaration(node: &Node) -> bool {
    crate::ast::is_variable_declaration(node)
        && node
            .expression()
            .map(|e| e.kind == SyntaxKind::ThisKeyword)
            .unwrap_or(false)
}

pub fn is_declaration_readonly(declaration: &Arc<Node>) -> bool {
    get_combined_modifier_flags(declaration).contains(ModifierFlags::Readonly)

}

pub fn get_binding_element_property_name(node: &Node) -> Option<Arc<Node>> {
    node.name().cloned()
}

pub fn is_valid_number_string(s: &str, round_trip_only: bool) -> bool {
    if s.is_empty() {
        return false;
    }
    let n = crate::jsnum::Number::from_string(s);
    !n.is_nan() && !n.is_inf() && (!round_trip_only || n.to_string() == s)
}

pub fn is_valid_big_int_string(_s: &str, _round_trip_only: bool) -> bool {

    false
}

pub fn is_valid_es_symbol_declaration(node: &Node) -> bool {

    let _ = node;
    false
}

pub fn is_variable_declaration_in_variable_statement(node: &Node) -> bool {
    node.parent
        .as_ref()
        .map(|p| is_variable_declaration_list(p))
        .unwrap_or(false)
        && node
            .parent
            .as_ref()
            .and_then(|p| p.parent.as_ref())
            .map(|gp| is_variable_statement(gp))
            .unwrap_or(false)
}

pub fn is_in_ambient_or_type_node(node: &Node) -> bool {
    if node.flags.contains(NodeFlags::Ambient) {
        return true;
    }

    false
}

pub fn is_private_within_ambient(node: &Node) -> bool {
    (crate::ast::has_syntactic_modifier(node, ModifierFlags::Private))
        && node.flags.contains(NodeFlags::Ambient)
}

pub fn pseudo_big_int_to_string(value: &crate::jsnum::PseudoBigInt) -> String {
    value.to_string()
}

pub fn value_to_string(value: &LiteralValue) -> String {
    match value {
        LiteralValue::String(s) => format!("\"{}\"", s),
        LiteralValue::Number(n) => n.to_string(),
        LiteralValue::Boolean(b) => b.to_string(),
        LiteralValue::BigInt(b) => format!("{}n", b),
        LiteralValue::None => String::new(),
    }
}

pub fn get_non_rest_parameter_count(sig: &Signature) -> usize {
    let has_rest = sig.flags.contains(SignatureFlags::HasRestParameter);
    sig.parameters.len() - if has_rest { 1 } else { 0 }
}

pub fn contains_non_missing_undefined_type(t: &Type) -> bool {
    if t.flags.contains(TypeFlags::Union) {
        if let TypeData::Union(u) = &t.data {
            if let Some(first) = u.union_or_intersection.types.first() {
                return first.flags.contains(TypeFlags::Undefined);
            }
        }
        false
    } else {
        t.flags.contains(TypeFlags::Undefined)
    }
}

pub fn try_get_property_access_or_identifier_to_string(expr: &Node) -> String {

    if crate::ast::is_identifier(expr) {
        return expr.text().to_string();
    }
    String::new()
}

pub fn get_set_accessor_value_parameter(accessor: &Node) -> Option<Arc<Node>> {

    let _ = accessor;
    None
}

pub fn get_super_container(node: &Node, _stop_on_functions: bool) -> Option<Arc<Node>> {

    node.parent.clone()
}

pub fn get_alias_declaration_from_name(node: &Node) -> Option<Arc<Node>> {

    let _ = node;
    None
}

pub fn get_containing_object_literal(f: &Node) -> Option<Arc<Node>> {

    let _ = f;
    None
}

pub fn is_import_type_qualifier_part(node: &Node) -> Option<Arc<Node>> {

    let _ = node;
    None
}

pub fn is_in_name_of_expression_with_type_arguments(node: &Node) -> bool {

    let _ = node;
    false
}

pub fn is_in_right_side_of_import_or_export_assignment(node: &Node) -> bool {

    let _ = node;
    false
}

pub fn is_class_instance_property(node: &Node) -> bool {

    node.parent
        .as_ref()
        .map(|p| {
            crate::ast::is_class_like(p)
                && crate::ast::is_property_declaration(node)
                && !crate::ast::has_accessor_modifier(node)
        })
        .unwrap_or(false)
}

pub fn is_this_initialized_object_binding_expression(node: &Node) -> bool {

    let _ = node;
    false
}

pub fn get_members_of_declaration(node: &Node) -> Vec<Arc<Node>> {

    let _ = node;
    Vec::new()
}

pub fn expression_result_is_unused(node: &Node) -> bool {

    let _ = node;
    false
}

pub fn for_each_yield_expression(body: &Node, _visitor: impl Fn(&Node)) {

    let _ = body;
}

pub fn is_jsdoc_optional_parameter(_node: &Node) -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::OnceLock;

    fn make_intrinsic(flags: TypeFlags, name: &str) -> Type {
        Type::new(
            flags,
            TypeData::Intrinsic(IntrinsicTypeData {
                intrinsic_name: name.to_string(),
            }),
        )
    }

    fn make_string_literal(s: &str) -> Type {
        Type::new(
            TypeFlags::StringLiteral,
            TypeData::Literal(LiteralTypeData {
                value: LiteralValue::String(s.to_string()),
                fresh_type: OnceLock::new(),
                regular_type: OnceLock::new(),
            }),
        )
    }

    fn make_number_literal(n: f64) -> Type {
        Type::new(
            TypeFlags::NumberLiteral,
            TypeData::Literal(LiteralTypeData {
                value: LiteralValue::Number(crate::jsnum::Number(n)),
                fresh_type: OnceLock::new(),
                regular_type: OnceLock::new(),
            }),
        )
    }

    #[test]
    fn test_type_flag_helpers() {
        let any = make_intrinsic(TypeFlags::Any, "any");
        assert!(is_any_or_unknown_type(&any));
        assert!(is_intrinsic_type(&any));
        assert!(is_singleton_type(&any));
        assert!(!is_nullable_type(&any));

        let null = make_intrinsic(TypeFlags::Null, "null");
        assert!(is_nullable_type(&null));
        assert!(!is_definitely_non_nullable_type(&null));

        let str = make_intrinsic(TypeFlags::String, "string");
        assert!(is_string_like_type(&str));
        assert!(is_primitive_type(&str));
        assert!(is_definitely_non_nullable_type(&str));
    }

    #[test]
    fn test_literal_type_helpers() {
        let s = make_string_literal("hello");
        assert!(is_literal_type(&s));
        assert!(is_string_like_type(&s));
        assert!(is_unit_type(&s));
        assert!(!is_number_like_type(&s));

        let n = make_number_literal(42.0);
        assert!(is_literal_type(&n));
        assert!(is_number_like_type(&n));
        assert!(is_unit_type(&n));
    }

    #[test]
    fn test_get_property_name_from_type() {
        let s = make_string_literal("foo");
        assert_eq!(get_property_name_from_type(&s), "foo");

        let n = make_number_literal(42.0);
        assert_eq!(get_property_name_from_type(&n), "42");
    }

    #[test]
    fn test_type_to_string() {
        let any = make_intrinsic(TypeFlags::Any, "any");
        assert_eq!(type_to_string(&any), "any");

        let str = make_intrinsic(TypeFlags::String, "string");
        assert_eq!(type_to_string(&str), "string");

        let s = make_string_literal("hello");
        assert_eq!(type_to_string(&s), "\"hello\"");

        let n = make_number_literal(42.0);
        assert_eq!(type_to_string(&n), "42");
    }

    #[test]
    fn test_can_have_locals() {
        assert!(can_have_locals(SyntaxKind::SourceFile));
        assert!(can_have_locals(SyntaxKind::FunctionDeclaration));
        assert!(can_have_locals(SyntaxKind::Block));
        assert!(can_have_locals(SyntaxKind::ForStatement));
        assert!(!can_have_locals(SyntaxKind::VariableDeclaration));
        assert!(!can_have_locals(SyntaxKind::IfStatement));
    }

    #[test]
    fn test_token_is_identifier_or_keyword() {
        assert!(token_is_identifier_or_keyword(SyntaxKind::Identifier));

        assert!(!token_is_identifier_or_keyword(SyntaxKind::PlusToken));
        assert!(!token_is_identifier_or_keyword(SyntaxKind::MinusToken));
    }

    #[test]
    fn test_exhaustive_state() {
        assert!(ExhaustiveState::True.is_true());
        assert!(ExhaustiveState::False.is_false());
        assert!(ExhaustiveState::Unknown.is_unknown());
        assert!(ExhaustiveState::Computing.is_computing());
        assert!(!ExhaustiveState::True.is_false());
    }

    #[test]
    fn test_specific_type_checks() {
        let any = make_intrinsic(TypeFlags::Any, "any");
        assert!(is_type_any(&any));
        assert!(!is_type_unknown(&any));

        let unknown = make_intrinsic(TypeFlags::Unknown, "unknown");
        assert!(is_type_unknown(&unknown));
        assert!(!is_type_any(&unknown));

        let never = make_intrinsic(TypeFlags::Never, "never");
        assert!(is_type_never(&never));

        let void_t = make_intrinsic(TypeFlags::Void, "void");
        assert!(is_type_void(&void_t));

        let string_t = make_intrinsic(TypeFlags::String, "string");
        assert!(is_type_string(&string_t));
        assert!(!is_type_number(&string_t));

        let number_t = make_intrinsic(TypeFlags::Number, "number");
        assert!(is_type_number(&number_t));

        let bool_t = make_intrinsic(TypeFlags::Boolean, "boolean");
        assert!(is_type_boolean(&bool_t));

        let bigint_t = make_intrinsic(TypeFlags::BigInt, "bigint");
        assert!(is_type_bigint(&bigint_t));

        let error = make_intrinsic(TypeFlags::Any, "error");
        assert!(is_type_error(&error));
        assert!(!is_type_error(&any));
    }

    #[test]
    fn test_symbol_name_helpers() {
        assert!(is_computed_property_name("[foo]"));
        assert!(!is_computed_property_name("foo"));
        assert!(is_internal_symbol_name(
            crate::ast::INTERNAL_SYMBOL_NAME_PREFIX
        ));
        assert!(!is_internal_symbol_name("normalName"));
    }

    #[test]
    fn test_numeric_literal_helpers() {
        assert!(is_numeric_literal_name("42"));
        assert!(is_numeric_literal_name("3.14"));
        assert!(!is_numeric_literal_name("hello"));
        assert_eq!(get_numeric_literal_name("42"), "42");
    }
}
