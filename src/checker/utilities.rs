//! Checker utility functions.
//!
//! Ported from `internal/checker/utilities.go`. Contains standalone helper
//! functions used throughout the checker. Functions that require `Checker`
//! state are methods on `Checker` in `checker.rs`.

use crate::ast::{Symbol, SyntaxKind};

use super::types::*;

// ────────────────────────────────────────────────────────────────────────────
// Token/Node helpers
// ────────────────────────────────────────────────────────────────────────────

/// Whether a syntax kind is an identifier or keyword (>= Identifier in the
/// SyntaxKind enum, matching Go's ordering).
pub fn token_is_identifier_or_keyword(kind: SyntaxKind) -> bool {
    (kind as u32) >= (SyntaxKind::Identifier as u32)
}

/// Whether a syntax kind is an identifier, keyword, or `>` token.
pub fn token_is_identifier_or_keyword_or_greater_than(kind: SyntaxKind) -> bool {
    kind == SyntaxKind::GreaterThanToken || token_is_identifier_or_keyword(kind)
}

/// Whether a node kind can contain local variable declarations.
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

// ────────────────────────────────────────────────────────────────────────────
// Type flag helpers
// ────────────────────────────────────────────────────────────────────────────

/// Whether a type is `any` or `unknown`.
pub fn is_any_or_unknown_type(t: &Type) -> bool {
    t.flags.intersects(TYPE_FLAGS_ANY_OR_UNKNOWN)
}

/// Whether a type is `null` or `undefined`.
pub fn is_nullable_type(t: &Type) -> bool {
    t.flags.intersects(TYPE_FLAGS_NULLABLE)
}

/// Whether a type is a literal type (string, number, bigint, or boolean literal).
pub fn is_literal_type(t: &Type) -> bool {
    t.flags.intersects(TYPE_FLAGS_LITERAL)
}

/// Whether a type is a unit type (enum, literal, unique symbol, null, or undefined).
pub fn is_unit_type(t: &Type) -> bool {
    t.flags.intersects(TYPE_FLAGS_UNIT)
}

/// Whether a type is a string-like type (string, string literal, template literal, string mapping).
pub fn is_string_like_type(t: &Type) -> bool {
    t.flags.intersects(TYPE_FLAGS_STRING_LIKE)
}

/// Whether a type is a number-like type (number, number literal, enum).
pub fn is_number_like_type(t: &Type) -> bool {
    t.flags.intersects(TYPE_FLAGS_NUMBER_LIKE)
}

/// Whether a type is a boolean-like type (boolean, boolean literal).
pub fn is_boolean_like_type(t: &Type) -> bool {
    t.flags.intersects(TYPE_FLAGS_BOOLEAN_LIKE)
}

/// Whether a type is an enum-like type (enum, enum literal).
pub fn is_enum_like_type(t: &Type) -> bool {
    t.flags.intersects(TYPE_FLAGS_ENUM_LIKE)
}

/// Whether a type is a primitive type.
pub fn is_primitive_type(t: &Type) -> bool {
    t.flags.intersects(TYPE_FLAGS_PRIMITIVE)
}

/// Whether a type is definitely falsy.
pub fn is_definitely_falsy_type(t: &Type) -> bool {
    t.flags.intersects(TYPE_FLAGS_DEFINITELY_FALSY)
}

/// Whether a type is possibly falsy.
pub fn is_possibly_falsy_type(t: &Type) -> bool {
    t.flags.intersects(TYPE_FLAGS_POSSIBLY_FALSY)
}

/// Whether a type is definitely non-nullable.
pub fn is_definitely_non_nullable_type(t: &Type) -> bool {
    t.flags.intersects(TYPE_FLAGS_DEFINITELY_NON_NULLABLE)
}

/// Whether a type is a structured type (object, union, or intersection).
pub fn is_structured_type(t: &Type) -> bool {
    t.flags.intersects(TYPE_FLAGS_STRUCTURED_TYPE)
}

/// Whether a type is an instantiable type (type variable, conditional, substitution, index, template literal, string mapping).
pub fn is_instantiable_type(t: &Type) -> bool {
    t.flags.intersects(TYPE_FLAGS_INSTANTIABLE)
}

/// Whether a type is a type variable (type parameter or indexed access).
pub fn is_type_variable(t: &Type) -> bool {
    t.flags.intersects(TYPE_FLAGS_TYPE_VARIABLE)
}

/// Whether a type is a union or intersection.
pub fn is_union_or_intersection_type(t: &Type) -> bool {
    t.flags.intersects(TYPE_FLAGS_UNION_OR_INTERSECTION)
}

/// Whether a type is an object type (has ObjectFlags).
pub fn is_object_flags_type(t: &Type) -> bool {
    t.flags.intersects(TYPE_FLAGS_OBJECT_FLAGS_TYPE)
}

/// Whether a type is a freshable type (enum or literal).
pub fn is_freshable_type(t: &Type) -> bool {
    t.flags.intersects(TYPE_FLAGS_FRESHABLE)
}

/// Whether a type is a singleton (one of the intrinsic singletons like any, string, etc.).
pub fn is_singleton_type(t: &Type) -> bool {
    t.flags.intersects(TYPE_FLAGS_SINGLETON)
}

/// Whether a type is narrowable.
pub fn is_narrowable_type(t: &Type) -> bool {
    t.flags.intersects(TYPE_FLAGS_NARROWABLE)
}

/// Whether a type is an intrinsic type.
pub fn is_intrinsic_type(t: &Type) -> bool {
    t.flags.intersects(TYPE_FLAGS_INTRINSIC)
}

/// Whether a type is void-like (void or undefined).
pub fn is_void_like_type(t: &Type) -> bool {
    t.flags.intersects(TYPE_FLAGS_VOID_LIKE)
}

// ────────────────────────────────────────────────────────────────────────────
// Object flag helpers
// ────────────────────────────────────────────────────────────────────────────

/// Whether a type is a class.
pub fn is_class_type(t: &Type) -> bool {
    t.object_flags.contains(ObjectFlags::Class)
}

/// Whether a type is an interface.
pub fn is_interface_type(t: &Type) -> bool {
    t.object_flags.contains(ObjectFlags::Interface)
}

/// Whether a type is a class or interface.
pub fn is_class_or_interface_type(t: &Type) -> bool {
    t.object_flags.intersects(OBJECT_FLAGS_CLASS_OR_INTERFACE)
}

/// Whether a type is a reference (instantiated generic type).
pub fn is_type_reference(t: &Type) -> bool {
    t.object_flags.contains(ObjectFlags::Reference)
}

/// Whether a type is an anonymous object type.
pub fn is_anonymous_type(t: &Type) -> bool {
    t.object_flags.contains(ObjectFlags::Anonymous)
}

/// Whether a type is a mapped type.
pub fn is_mapped_type(t: &Type) -> bool {
    t.object_flags.contains(ObjectFlags::Mapped)
}

/// Whether a type is a tuple type.
pub fn is_tuple_type(t: &Type) -> bool {
    // A tuple type is a reference to an interface type with the Tuple flag
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

/// Whether a type is an evolving array type.
pub fn is_evolving_array_type(t: &Type) -> bool {
    t.flags.contains(TypeFlags::Object) && t.object_flags.contains(ObjectFlags::EvolvingArray)
}

/// Whether a type is a fresh object literal.
pub fn is_fresh_object_literal_type(t: &Type) -> bool {
    t.flags.contains(TypeFlags::Object) && t.object_flags.contains(ObjectFlags::FreshLiteral)
}

/// Whether a type is an object literal type.
pub fn is_object_literal_type(t: &Type) -> bool {
    t.flags.contains(TypeFlags::Object) && t.object_flags.contains(ObjectFlags::ObjectLiteral)
}

// ────────────────────────────────────────────────────────────────────────────
// Property name helpers
// ────────────────────────────────────────────────────────────────────────────

/// Whether a type can be used as a property name.
pub fn is_type_usable_as_property_name(t: &Type) -> bool {
    t.flags
        .intersects(TYPE_FLAGS_STRING_OR_NUMBER_LITERAL_OR_UNIQUE | TypeFlags::ESSymbol)
}

/// Get the property name string from a type.
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

// ────────────────────────────────────────────────────────────────────────────
// Type display helpers (basic)
// ────────────────────────────────────────────────────────────────────────────

/// Convert a type to a display string (basic implementation).
///
/// Full type-to-string conversion requires the NodeBuilder, which is
/// ported separately. This provides a minimal version for intrinsic and
/// literal types.
pub fn type_to_string(t: &Type) -> String {
    // Intrinsic types
    if let Some(name) = t.intrinsic_name() {
        return name.to_string();
    }

    // Literal types
    if let Some(val) = t.literal_value() {
        return val.to_string();
    }

    // Union types
    if t.is_union() {
        if let Some(types) = t.types() {
            let parts: Vec<String> = types.iter().map(|ty| type_to_string(ty)).collect();
            return parts.join(" | ");
        }
    }

    // Intersection types
    if t.is_intersection() {
        if let Some(types) = t.types() {
            let parts: Vec<String> = types.iter().map(|ty| type_to_string(ty)).collect();
            return parts.join(" & ");
        }
    }

    // Type parameter
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

    // Object types with a symbol
    if let Some(sym) = &t.symbol {
        return sym.name.clone();
    }

    // Fallback
    if t.flags.contains(TypeFlags::Never) {
        return "never".to_string();
    }
    if t.flags.contains(TypeFlags::Object) {
        return "object".to_string();
    }

    "<unknown type>".to_string()
}

// ────────────────────────────────────────────────────────────────────────────
// AssignmentKind
// ────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AssignmentKind {
    #[default]
    None,
    Definite,
    Compound,
}

// ────────────────────────────────────────────────────────────────────────────
// ExhaustiveState helpers
// ────────────────────────────────────────────────────────────────────────────

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

// ────────────────────────────────────────────────────────────────────────────
// Symbol helpers
// ────────────────────────────────────────────────────────────────────────────

/// Whether a symbol is a late-bound symbol.
pub fn is_late_bound_symbol(_symbol: &Symbol) -> bool {
    // TODO: check if the symbol has late binding links
    false
}

/// Whether a symbol represents a value.
pub fn is_value_symbol(symbol: &Symbol) -> bool {
    symbol.flags.intersects(crate::ast::SymbolFlags::VALUE)
}

/// Whether a symbol represents a type.
pub fn is_type_symbol(symbol: &Symbol) -> bool {
    symbol.flags.intersects(crate::ast::SymbolFlags::TYPE)
}

/// Whether a symbol represents a namespace.
pub fn is_namespace_symbol(symbol: &Symbol) -> bool {
    symbol.flags.intersects(crate::ast::SymbolFlags::NAMESPACE)
}

/// Whether a symbol is optional.
pub fn is_optional_symbol(symbol: &Symbol) -> bool {
    symbol.flags.intersects(crate::ast::SymbolFlags::Optional)
}

/// Whether a symbol is a class member.
pub fn is_class_member_symbol(symbol: &Symbol) -> bool {
    symbol
        .flags
        .intersects(crate::ast::SymbolFlags::CLASS_MEMBER)
}

// ────────────────────────────────────────────────────────────────────────────
// Specific type check helpers
// ────────────────────────────────────────────────────────────────────────────

/// Whether a type is `any`.
pub fn is_type_any(t: &Type) -> bool {
    t.flags.contains(TypeFlags::Any)
}

/// Whether a type is `unknown`.
pub fn is_type_unknown(t: &Type) -> bool {
    t.flags.contains(TypeFlags::Unknown)
}

/// Whether a type is `never`.
pub fn is_type_never(t: &Type) -> bool {
    t.flags.contains(TypeFlags::Never)
}

/// Whether a type is `void`.
pub fn is_type_void(t: &Type) -> bool {
    t.flags.contains(TypeFlags::Void)
}

/// Whether a type is `undefined`.
pub fn is_type_undefined(t: &Type) -> bool {
    t.flags.contains(TypeFlags::Undefined)
}

/// Whether a type is `null`.
pub fn is_type_null(t: &Type) -> bool {
    t.flags.contains(TypeFlags::Null)
}

/// Whether a type is `string`.
pub fn is_type_string(t: &Type) -> bool {
    t.flags.contains(TypeFlags::String)
}

/// Whether a type is `number`.
pub fn is_type_number(t: &Type) -> bool {
    t.flags.contains(TypeFlags::Number)
}

/// Whether a type is `boolean`.
pub fn is_type_boolean(t: &Type) -> bool {
    t.flags.contains(TypeFlags::Boolean)
}

/// Whether a type is `bigint`.
pub fn is_type_bigint(t: &Type) -> bool {
    t.flags.contains(TypeFlags::BigInt)
}

/// Whether a type is `symbol`.
pub fn is_symbol_type(t: &Type) -> bool {
    t.flags.contains(TypeFlags::ESSymbol)
}

/// Whether a type is `object` (non-primitive).
pub fn is_type_non_primitive(t: &Type) -> bool {
    t.flags.contains(TypeFlags::NonPrimitive)
}

/// Whether a type is the error type (has the `error` intrinsic name).
pub fn is_type_error(t: &Type) -> bool {
    t.intrinsic_name() == Some("error")
}

/// Whether a type is a fresh literal type.
pub fn is_fresh_literal_type(t: &Type) -> bool {
    if let TypeData::Literal(lit) = &t.data {
        lit.fresh_type.get().is_some()
    } else {
        false
    }
}

/// Whether a type is an array type (reference to `Array<T>`).
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

/// Whether a type is an array or tuple type.
pub fn is_array_or_tuple_type(t: &Type) -> bool {
    is_array_type(t) || is_tuple_type(t)
}

// ────────────────────────────────────────────────────────────────────────────
// Symbol name helpers
// ────────────────────────────────────────────────────────────────────────────

/// Whether a symbol name is a computed property name (starts with `[`).
pub fn is_computed_property_name(name: &str) -> bool {
    name.starts_with('[')
}

/// Whether a symbol name is an internal name (starts with the internal prefix).
pub fn is_internal_symbol_name(name: &str) -> bool {
    name.starts_with(crate::ast::INTERNAL_SYMBOL_NAME_PREFIX)
}

// ────────────────────────────────────────────────────────────────────────────
// Numeric literal helpers
// ────────────────────────────────────────────────────────────────────────────

/// Whether a string represents a numeric literal.
pub fn is_numeric_literal_name(name: &str) -> bool {
    name.parse::<f64>().is_ok()
}

/// Convert a numeric string to a property name (e.g. "42" → "42").
pub fn get_numeric_literal_name(name: &str) -> String {
    if let Ok(n) = name.parse::<f64>() {
        crate::jsnum::Number(n).to_string()
    } else {
        name.to_string()
    }
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
        // Keywords come after Identifier in the SyntaxKind enum
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
