#![allow(unused_imports)]

use super::*;

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
