#![allow(unused_imports)]

use super::*;

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
