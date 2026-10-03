#![allow(unused_imports)]

use crate::checker::utilities::*;

pub fn is_optional_symbol(symbol: &Symbol) -> bool { ::tsox_core::fntrace::enter("is_optional_symbol"); 
    symbol
        .flags
        .intersects(tsox_frontend::ast::SymbolFlags::Optional)
}

pub fn is_class_member_symbol(symbol: &Symbol) -> bool { ::tsox_core::fntrace::enter("is_class_member_symbol"); 
    symbol
        .flags
        .intersects(tsox_frontend::ast::SymbolFlags::CLASS_MEMBER)
}

pub fn is_type_any(t: &Type) -> bool { ::tsox_core::fntrace::enter("is_type_any"); 
    t.flags.contains(TypeFlags::Any)
}

pub fn is_type_unknown(t: &Type) -> bool { ::tsox_core::fntrace::enter("is_type_unknown"); 
    t.flags.contains(TypeFlags::Unknown)
}

pub fn is_type_never(t: &Type) -> bool { ::tsox_core::fntrace::enter("is_type_never"); 
    t.flags.contains(TypeFlags::Never)
}

pub fn is_type_void(t: &Type) -> bool { ::tsox_core::fntrace::enter("is_type_void"); 
    t.flags.contains(TypeFlags::Void)
}

pub fn is_type_undefined(t: &Type) -> bool { ::tsox_core::fntrace::enter("is_type_undefined"); 
    t.flags.contains(TypeFlags::Undefined)
}

pub fn is_type_null(t: &Type) -> bool { ::tsox_core::fntrace::enter("is_type_null"); 
    t.flags.contains(TypeFlags::Null)
}

pub fn is_type_string(t: &Type) -> bool { ::tsox_core::fntrace::enter("is_type_string"); 
    t.flags.contains(TypeFlags::String)
}

pub fn is_type_number(t: &Type) -> bool { ::tsox_core::fntrace::enter("is_type_number"); 
    t.flags.contains(TypeFlags::Number)
}

pub fn is_type_boolean(t: &Type) -> bool { ::tsox_core::fntrace::enter("is_type_boolean"); 
    t.flags.contains(TypeFlags::Boolean)
}

pub fn is_type_bigint(t: &Type) -> bool { ::tsox_core::fntrace::enter("is_type_bigint"); 
    t.flags.contains(TypeFlags::BigInt)
}

pub fn is_symbol_type(t: &Type) -> bool { ::tsox_core::fntrace::enter("is_symbol_type"); 
    t.flags.contains(TypeFlags::ESSymbol)
}

pub fn is_type_non_primitive(t: &Type) -> bool { ::tsox_core::fntrace::enter("is_type_non_primitive"); 
    t.flags.contains(TypeFlags::NonPrimitive)
}

pub fn is_type_error(t: &Type) -> bool { ::tsox_core::fntrace::enter("is_type_error"); 
    t.intrinsic_name() == Some("error")
}

pub fn is_fresh_literal_type(t: &Type) -> bool { ::tsox_core::fntrace::enter("is_fresh_literal_type"); 
    if let TypeData::Literal(lit) = &t.data {
        lit.regular_type.get().is_some()
    } else {
        false
    }
}

pub fn is_array_type(t: &Type) -> bool { ::tsox_core::fntrace::enter("is_array_type"); 
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

pub fn is_array_or_tuple_type(t: &Type) -> bool { ::tsox_core::fntrace::enter("is_array_or_tuple_type"); 
    is_array_type(t) || is_tuple_type(t)
}

pub fn is_computed_property_name(name: &str) -> bool { ::tsox_core::fntrace::enter("is_computed_property_name"); 
    name.starts_with('[')
}

pub fn is_internal_symbol_name(name: &str) -> bool { ::tsox_core::fntrace::enter("is_internal_symbol_name"); 
    name.starts_with(tsox_frontend::ast::INTERNAL_SYMBOL_NAME_PREFIX)
}

pub fn is_numeric_literal_name(name: &str) -> bool { ::tsox_core::fntrace::enter("is_numeric_literal_name"); 
    name.parse::<f64>().is_ok()
}

pub fn get_numeric_literal_name(name: &str) -> String { ::tsox_core::fntrace::enter("get_numeric_literal_name"); 
    if let Ok(n) = name.parse::<f64>() {
        tsox_core::jsnum::Number(n).to_string()
    } else {
        name.to_string()
    }
}

pub fn is_exponentiation_operator(kind: SyntaxKind) -> bool { ::tsox_core::fntrace::enter("is_exponentiation_operator"); 
    kind == SyntaxKind::AsteriskAsteriskToken
}

pub fn is_multiplicative_operator(kind: SyntaxKind) -> bool { ::tsox_core::fntrace::enter("is_multiplicative_operator"); 
    matches!(
        kind,
        SyntaxKind::AsteriskToken | SyntaxKind::SlashToken | SyntaxKind::PercentToken
    )
}

pub fn is_multiplicative_operator_or_higher(kind: SyntaxKind) -> bool { ::tsox_core::fntrace::enter("is_multiplicative_operator_or_higher"); 
    is_exponentiation_operator(kind) || is_multiplicative_operator(kind)
}

pub fn is_additive_operator(kind: SyntaxKind) -> bool { ::tsox_core::fntrace::enter("is_additive_operator"); 
    matches!(kind, SyntaxKind::PlusToken | SyntaxKind::MinusToken)
}

pub fn is_additive_operator_or_higher(kind: SyntaxKind) -> bool { ::tsox_core::fntrace::enter("is_additive_operator_or_higher"); 
    is_additive_operator(kind) || is_multiplicative_operator_or_higher(kind)
}

pub fn is_shift_operator(kind: SyntaxKind) -> bool { ::tsox_core::fntrace::enter("is_shift_operator"); 
    matches!(
        kind,
        SyntaxKind::LessThanLessThanToken
            | SyntaxKind::GreaterThanGreaterThanToken
            | SyntaxKind::GreaterThanGreaterThanGreaterThanToken
    )
}

pub fn is_shift_operator_or_higher(kind: SyntaxKind) -> bool { ::tsox_core::fntrace::enter("is_shift_operator_or_higher"); 
    is_shift_operator(kind) || is_additive_operator_or_higher(kind)
}

pub fn is_relational_operator(kind: SyntaxKind) -> bool { ::tsox_core::fntrace::enter("is_relational_operator"); 
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

pub fn is_relational_operator_or_higher(kind: SyntaxKind) -> bool { ::tsox_core::fntrace::enter("is_relational_operator_or_higher"); 
    is_relational_operator(kind) || is_shift_operator_or_higher(kind)
}

pub fn is_equality_operator(kind: SyntaxKind) -> bool { ::tsox_core::fntrace::enter("is_equality_operator"); 
    matches!(
        kind,
        SyntaxKind::EqualsEqualsToken
            | SyntaxKind::EqualsEqualsEqualsToken
            | SyntaxKind::ExclamationEqualsToken
            | SyntaxKind::ExclamationEqualsEqualsToken
    )
}

pub fn is_equality_operator_or_higher(kind: SyntaxKind) -> bool { ::tsox_core::fntrace::enter("is_equality_operator_or_higher"); 
    is_equality_operator(kind) || is_relational_operator_or_higher(kind)
}

pub fn is_bitwise_operator(kind: SyntaxKind) -> bool { ::tsox_core::fntrace::enter("is_bitwise_operator"); 
    matches!(
        kind,
        SyntaxKind::AmpersandToken | SyntaxKind::BarToken | SyntaxKind::CaretToken
    )
}

pub fn is_bitwise_operator_or_higher(kind: SyntaxKind) -> bool { ::tsox_core::fntrace::enter("is_bitwise_operator_or_higher"); 
    is_bitwise_operator(kind) || is_equality_operator_or_higher(kind)
}

pub fn is_logical_operator_or_higher(kind: SyntaxKind) -> bool { ::tsox_core::fntrace::enter("is_logical_operator_or_higher"); 
    tsox_frontend::ast::is_logical_binary_operator(kind) || is_bitwise_operator_or_higher(kind)
}

pub fn is_assignment_operator_or_higher(kind: SyntaxKind) -> bool { ::tsox_core::fntrace::enter("is_assignment_operator_or_higher"); 
    kind == SyntaxKind::QuestionQuestionToken
        || is_logical_operator_or_higher(kind)
        || tsox_frontend::ast::is_assignment_operator(kind)
}

pub fn is_binary_operator(kind: SyntaxKind) -> bool { ::tsox_core::fntrace::enter("is_binary_operator"); 
    is_assignment_operator_or_higher(kind) || kind == SyntaxKind::CommaToken
}

pub fn has_override_modifier(node: &Node) -> bool { ::tsox_core::fntrace::enter("has_override_modifier"); 
    tsox_frontend::ast::has_syntactic_modifier(node, ModifierFlags::Override)
}

pub fn has_async_modifier(node: &Node) -> bool { ::tsox_core::fntrace::enter("has_async_modifier"); 
    tsox_frontend::ast::has_syntactic_modifier(node, ModifierFlags::Async)
}

pub fn get_selected_modifier_flags(node: &Node, flags: ModifierFlags) -> ModifierFlags { ::tsox_core::fntrace::enter("get_selected_modifier_flags"); 
    node.syntactic_modifier_flags() & flags
}

pub fn has_readonly_modifier(node: &Node) -> bool { ::tsox_core::fntrace::enter("has_readonly_modifier"); 
    tsox_frontend::ast::has_syntactic_modifier(node, ModifierFlags::Readonly)
}

pub fn is_infinity_or_nan_string(name: &str) -> bool { ::tsox_core::fntrace::enter("is_infinity_or_nan_string"); 
    name == "Infinity" || name == "-Infinity" || name == "NaN"
}

pub fn is_reserved_member_name(name: &str) -> bool { ::tsox_core::fntrace::enter("is_reserved_member_name"); 
    let bytes = name.as_bytes();
    bytes.len() >= 2 && bytes[0] == 0xFE && bytes[1] != b'@' && bytes[1] != b'#'
}

pub fn is_late_bound_name(name: &str) -> bool { ::tsox_core::fntrace::enter("is_late_bound_name"); 
    let bytes = name.as_bytes();
    bytes.len() >= 2 && bytes[0] == 0xFE && bytes[1] == b'@'
}

pub fn is_exclamation_token(node: &Node) -> bool { ::tsox_core::fntrace::enter("is_exclamation_token"); 
    node.kind == SyntaxKind::ExclamationToken
}

pub fn is_type_alias(node: &Node) -> bool { ::tsox_core::fntrace::enter("is_type_alias"); 
    matches!(
        node.kind,
        SyntaxKind::TypeAliasDeclaration | SyntaxKind::JSTypeAliasDeclaration
    )
}

pub fn is_literal_expression_of_object(node: &Node) -> bool { ::tsox_core::fntrace::enter("is_literal_expression_of_object"); 
    matches!(
        node.kind,
        SyntaxKind::ObjectLiteralExpression
            | SyntaxKind::ArrayLiteralExpression
            | SyntaxKind::RegularExpressionLiteral
            | SyntaxKind::FunctionExpression
            | SyntaxKind::ClassExpression
    )
}

pub fn introduces_arguments_exotic_object(kind: SyntaxKind) -> bool { ::tsox_core::fntrace::enter("introduces_arguments_exotic_object"); 
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

pub fn node_starts_new_lexical_environment(kind: SyntaxKind) -> bool { ::tsox_core::fntrace::enter("node_starts_new_lexical_environment"); 
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
