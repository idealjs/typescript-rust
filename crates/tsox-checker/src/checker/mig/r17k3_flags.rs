use crate::checker::types_type_flags_instantiable_non_primitive::{
    AccessFlags, IndexFlags, ObjectFlags, ACCESS_FLAGS_PERSISTENT, OBJECT_FLAGS_IS_GENERIC_TYPE,
};
use crate::checker::types_type_id::{
    TypeFlags, TYPE_FLAGS_ANY_OR_UNKNOWN, TYPE_FLAGS_DEFINITELY_NON_NULLABLE,
    TYPE_FLAGS_DISJOINT_DOMAINS, TYPE_FLAGS_ES_SYMBOL_LIKE, TYPE_FLAGS_NULLABLE,
    TYPE_FLAGS_NUMBER_LIKE, TYPE_FLAGS_PRIMITIVE, TYPE_FLAGS_STRING_LIKE,
    TYPE_FLAGS_STRING_OR_NUMBER_LITERAL_OR_UNIQUE, TYPE_FLAGS_STRUCTURED_TYPE,
    TYPE_FLAGS_TYPE_VARIABLE, TYPE_FLAGS_UNION_OR_INTERSECTION,
};

pub(crate) use tsox_frontend::ast::INTERNAL_SYMBOL_NAME_DEFAULT as internal_symbol_name_default;
pub(crate) use tsox_frontend::ast::INTERNAL_SYMBOL_NAME_EXPORT_EQUALS as internal_symbol_name_export_equals;
pub(crate) use tsox_frontend::ast::INTERNAL_SYMBOL_NAME_INDEX as internal_symbol_name_index;
pub(crate) use tsox_frontend::ast::INTERNAL_SYMBOL_NAME_INSTANTIATION_EXPRESSION as internal_symbol_name_instantiation_expression;

impl TypeFlags {
    pub const ENUM_LITERAL: Self = Self::EnumLiteral;
    pub const STRING_LITERAL: Self = Self::StringLiteral;
    pub const NUMBER_LITERAL: Self = Self::NumberLiteral;
    pub const INDEX: Self = Self::Index;
    pub const UNION: Self = Self::Union;
    pub const INTERSECTION: Self = Self::Intersection;
    pub const SUBSTITUTION: Self = Self::Substitution;
    pub const ANY: Self = Self::Any;
    pub const UNKNOWN: Self = Self::Unknown;
    pub const NEVER: Self = Self::Never;
    pub const STRING: Self = Self::String;
    pub const NUMBER: Self = Self::Number;
    pub const BIGINT: Self = Self::BigInt;
    pub const BIGINT_LITERAL: Self = Self::BigIntLiteral;
    pub const BOOLEAN: Self = Self::Boolean;
    pub const ES_SYMBOL: Self = Self::ESSymbol;
    pub const UNIQUE_ES_SYMBOL: Self = Self::UniqueESSymbol;
    pub const VOID: Self = Self::Void;
    pub const UNDEFINED: Self = Self::Undefined;
    pub const OBJECT: Self = Self::Object;
    pub const NON_PRIMITIVE: Self = Self::NonPrimitive;
    pub const TEMPLATE_LITERAL: Self = Self::TemplateLiteral;
    pub const STRING_MAPPING: Self = Self::StringMapping;
    pub const UNION_OR_INTERSECTION: Self = TYPE_FLAGS_UNION_OR_INTERSECTION;
    pub const STRUCTURED_TYPE: Self = TYPE_FLAGS_STRUCTURED_TYPE;
    pub const STRING_LIKE: Self = TYPE_FLAGS_STRING_LIKE;
    pub const NUMBER_LIKE: Self = TYPE_FLAGS_NUMBER_LIKE;
    pub const ES_SYMBOL_LIKE: Self = TYPE_FLAGS_ES_SYMBOL_LIKE;
    pub const NULLABLE: Self = TYPE_FLAGS_NULLABLE;
    pub const DISJOINT_DOMAINS: Self = TYPE_FLAGS_DISJOINT_DOMAINS;
    pub const DEFINITELY_NON_NULLABLE: Self = TYPE_FLAGS_DEFINITELY_NON_NULLABLE;
    pub const TYPE_VARIABLE: Self = TYPE_FLAGS_TYPE_VARIABLE;
    pub const PRIMITIVE: Self = TYPE_FLAGS_PRIMITIVE;
    pub const ANY_OR_UNKNOWN: Self = TYPE_FLAGS_ANY_OR_UNKNOWN;
    pub const STRING_OR_NUMBER_LITERAL_OR_UNIQUE: Self = TYPE_FLAGS_STRING_OR_NUMBER_LITERAL_OR_UNIQUE;
    pub const INSTANTIABLE_NON_PRIMITIVE: Self = Self::from_bits_truncate(
        TYPE_FLAGS_TYPE_VARIABLE.bits() | Self::Conditional.bits() | Self::Substitution.bits(),
    );
    pub const INSTANTIABLE: Self = Self::from_bits_truncate(
        TYPE_FLAGS_TYPE_VARIABLE.bits()
            | Self::Conditional.bits()
            | Self::Substitution.bits()
            | Self::Index.bits()
            | Self::TemplateLiteral.bits()
            | Self::StringMapping.bits(),
    );
    pub const INCLUDES_MISSING_TYPE: Self = Self::TypeParameter;
    pub const INCLUDES_WILDCARD: Self = Self::IndexedAccess;
    pub const INCLUDES_EMPTY_OBJECT: Self = Self::Conditional;
    pub const INCLUDES_ERROR: Self = Self::Reserved2;
}

impl ObjectFlags {
    pub const IS_GENERIC_TYPE_COMPUTED: Self = Self::IsGenericTypeComputed;
    pub const IS_GENERIC_TYPE: Self = OBJECT_FLAGS_IS_GENERIC_TYPE;
    pub const IS_GENERIC_OBJECT_TYPE: Self = Self::IsGenericObjectType;
    pub const IS_GENERIC_INDEX_TYPE: Self = Self::IsGenericIndexType;
    pub const MAPPED: Self = Self::Mapped;
    pub const ANONYMOUS: Self = Self::Anonymous;
    pub const INSTANTIATION_EXPRESSION_TYPE: Self = Self::InstantiationExpressionType;
    pub const IS_CONSTRAINED_TYPE_VARIABLE: Self = Self::IsConstrainedTypeVariable;
}

impl AccessFlags {
    pub const EXPRESSION_POSITION: Self = Self::ExpressionPosition;
    pub const INCLUDE_UNDEFINED: Self = Self::IncludeUndefined;
    pub const PERSISTENT: Self = ACCESS_FLAGS_PERSISTENT;
    pub const SUPPRESS_NO_IMPLICIT_ANY_ERROR: Self = Self::SuppressNoImplicitAnyError;
    pub const WRITING: Self = Self::Writing;
    pub const CACHE_SYMBOL: Self = Self::CacheSymbol;
    pub const REPORT_DEPRECATED: Self = Self::ReportDeprecated;
}

impl IndexFlags {
    pub const NO_INDEX_SIGNATURES: Self = Self::NoIndexSignatures;
    pub const STRINGS_ONLY: Self = Self::StringsOnly;
}
