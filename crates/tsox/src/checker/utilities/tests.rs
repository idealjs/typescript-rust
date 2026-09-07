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
