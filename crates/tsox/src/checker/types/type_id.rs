#![allow(unused_imports)]

use super::*;

pub type TypeId = u32;

pub(crate) static NEXT_TYPE_ID: AtomicU32 = AtomicU32::new(1);

pub fn next_type_id() -> u32 {
    NEXT_TYPE_ID.fetch_add(1, Ordering::Relaxed)
}
pub type SignatureId = u32;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct ParseFlags: u32 {
        const None                   = 0;
        const Yield                  = 1 << 0;
        const Await                  = 1 << 1;
        const Type                   = 1 << 2;
        const IgnoreMissingOpenBrace = 1 << 4;
        const JSDoc                  = 1 << 5;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SignatureKind {
    #[default]
    Call,
    Construct,
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct ContextFlags: u32 {
        const None                 = 0;
        const Signature            = 1 << 0;
        const NoConstraints        = 1 << 1;
        const IgnoreNodeInferences = 1 << 2;
        const SkipBindingPatterns  = 1 << 3;
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct TypeFormatFlags: u32 {
        const None                               = 0;
        const NoTruncation                       = 1 << 0;
        const WriteArrayAsGenericType            = 1 << 1;
        const GenerateNamesForShadowedTypeParams = 1 << 2;
        const UseStructuralFallback              = 1 << 3;
        const WriteTypeArgumentsOfSignature      = 1 << 5;
        const UseFullyQualifiedType              = 1 << 6;
        const SuppressAnyReturnType              = 1 << 8;
        const MultilineObjectLiterals            = 1 << 10;
        const WriteClassExpressionAsTypeLiteral  = 1 << 11;
        const UseTypeOfFunction                  = 1 << 12;
        const OmitParameterModifiers             = 1 << 13;
        const UseAliasDefinedOutsideCurrentScope = 1 << 14;
        const UseSingleQuotesForStringLiteralType= 1 << 28;
        const NoTypeReduction                    = 1 << 29;
        const UseInstantiationExpressions        = 1 << 30;
        const OmitThisParameter                  = 1 << 25;
        const WriteCallStyleSignature            = 1 << 27;
        const AllowUniqueESSymbolType            = 1 << 20;
        const AddUndefined                       = 1 << 17;
        const WriteArrowStyleSignature           = 1 << 18;
        const InArrayType                        = 1 << 19;
        const InElementType                      = 1 << 21;
        const InFirstTypeArgument                = 1 << 22;
        const InTypeAlias                        = 1 << 23;
    }
}

pub const TYPE_FORMAT_FLAGS_NODE_BUILDER_MASK: TypeFormatFlags =
    TypeFormatFlags::from_bits_truncate(
        TypeFormatFlags::NoTruncation.bits()
            | TypeFormatFlags::WriteArrayAsGenericType.bits()
            | TypeFormatFlags::GenerateNamesForShadowedTypeParams.bits()
            | TypeFormatFlags::UseStructuralFallback.bits()
            | TypeFormatFlags::WriteTypeArgumentsOfSignature.bits()
            | TypeFormatFlags::UseFullyQualifiedType.bits()
            | TypeFormatFlags::SuppressAnyReturnType.bits()
            | TypeFormatFlags::MultilineObjectLiterals.bits()
            | TypeFormatFlags::WriteClassExpressionAsTypeLiteral.bits()
            | TypeFormatFlags::UseTypeOfFunction.bits()
            | TypeFormatFlags::OmitParameterModifiers.bits()
            | TypeFormatFlags::UseAliasDefinedOutsideCurrentScope.bits()
            | TypeFormatFlags::AllowUniqueESSymbolType.bits()
            | TypeFormatFlags::InTypeAlias.bits()
            | TypeFormatFlags::UseInstantiationExpressions.bits()
            | TypeFormatFlags::UseSingleQuotesForStringLiteralType.bits()
            | TypeFormatFlags::NoTypeReduction.bits()
            | TypeFormatFlags::OmitThisParameter.bits(),
    );

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct SymbolFormatFlags: u32 {
        const None                            = 0;
        const WriteTypeParametersOrArguments  = 1 << 0;
        const UseOnlyExternalAliasing         = 1 << 1;
        const AllowAnyNodeKind                = 1 << 2;
        const UseAliasDefinedOutsideCurrentScope = 1 << 3;
        const WriteComputedProps              = 1 << 4;
        const DoNotIncludeSymbolChain         = 1 << 5;
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct ExternalEmitHelpers: u32 {
        const Rest                               = 1 << 0;
        const Decorate                           = 1 << 1;
        const Metadata                           = 1 << 2;
        const Param                              = 1 << 3;
        const Awaiter                            = 1 << 4;
        const Await                              = 1 << 5;
        const AsyncGenerator                     = 1 << 6;
        const AsyncDelegator                     = 1 << 7;
        const AsyncValues                        = 1 << 8;
        const ExportStar                         = 1 << 9;
        const ImportStar                         = 1 << 10;
        const ImportDefault                      = 1 << 11;
        const MakeTemplateObject                 = 1 << 12;
        const ClassPrivateFieldGet               = 1 << 13;
        const ClassPrivateFieldSet               = 1 << 14;
        const ClassPrivateFieldIn                = 1 << 15;
        const SetFunctionName                    = 1 << 16;
        const PropKey                            = 1 << 17;
        const AddDisposableResourceAndDisposeResources = 1 << 18;
        const RewriteRelativeImportExtension     = 1 << 19;
        const ESDecorateAndRunInitializers       = Self::Decorate.bits();
    }
}

pub const EXTERNAL_EMIT_HELPERS_FIRST: ExternalEmitHelpers = ExternalEmitHelpers::Rest;
pub const EXTERNAL_EMIT_HELPERS_LAST: ExternalEmitHelpers =
    ExternalEmitHelpers::RewriteRelativeImportExtension;
pub const EXTERNAL_HELPERS_MODULE_NAME_TEXT: &str = "tslib";

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct TypeFlags: u32 {
        const None            = 0;
        const Any             = 1 << 0;
        const Unknown         = 1 << 1;
        const Undefined       = 1 << 2;
        const Null            = 1 << 3;
        const Void            = 1 << 4;
        const String          = 1 << 5;
        const Number          = 1 << 6;
        const BigInt          = 1 << 7;
        const Boolean         = 1 << 8;
        const ESSymbol        = 1 << 9;
        const StringLiteral   = 1 << 10;
        const NumberLiteral   = 1 << 11;
        const BigIntLiteral   = 1 << 12;
        const BooleanLiteral  = 1 << 13;
        const UniqueESSymbol  = 1 << 14;
        const EnumLiteral     = 1 << 15;
        const Enum            = 1 << 16;
        const NonPrimitive    = 1 << 17;
        const Never           = 1 << 18;
        const TypeParameter   = 1 << 19;
        const Object          = 1 << 20;
        const Index           = 1 << 21;
        const TemplateLiteral = 1 << 22;
        const StringMapping   = 1 << 23;
        const Substitution    = 1 << 24;
        const IndexedAccess   = 1 << 25;
        const Conditional     = 1 << 26;
        const Union           = 1 << 27;
        const Intersection    = 1 << 28;
        const Reserved1       = 1 << 29;
        const Reserved2       = 1 << 30;
        const Reserved3       = 1 << 31;
    }
}

pub const TYPE_FLAGS_ANY_OR_UNKNOWN: TypeFlags =
    TypeFlags::from_bits_truncate(TypeFlags::Any.bits() | TypeFlags::Unknown.bits());
pub const TYPE_FLAGS_NULLABLE: TypeFlags =
    TypeFlags::from_bits_truncate(TypeFlags::Undefined.bits() | TypeFlags::Null.bits());
pub const TYPE_FLAGS_LITERAL: TypeFlags = TypeFlags::from_bits_truncate(
    TypeFlags::StringLiteral.bits()
        | TypeFlags::NumberLiteral.bits()
        | TypeFlags::BigIntLiteral.bits()
        | TypeFlags::BooleanLiteral.bits(),
);
pub const TYPE_FLAGS_UNIT: TypeFlags = TypeFlags::from_bits_truncate(
    TypeFlags::Enum.bits()
        | TYPE_FLAGS_LITERAL.bits()
        | TypeFlags::UniqueESSymbol.bits()
        | TYPE_FLAGS_NULLABLE.bits(),
);
pub const TYPE_FLAGS_FRESHABLE: TypeFlags =
    TypeFlags::from_bits_truncate(TypeFlags::Enum.bits() | TYPE_FLAGS_LITERAL.bits());
pub const TYPE_FLAGS_STRING_OR_NUMBER_LITERAL: TypeFlags = TypeFlags::from_bits_truncate(
    TypeFlags::StringLiteral.bits() | TypeFlags::NumberLiteral.bits(),
);
pub const TYPE_FLAGS_STRING_OR_NUMBER_LITERAL_OR_UNIQUE: TypeFlags = TypeFlags::from_bits_truncate(
    TypeFlags::StringLiteral.bits()
        | TypeFlags::NumberLiteral.bits()
        | TypeFlags::UniqueESSymbol.bits(),
);
pub const TYPE_FLAGS_DEFINITELY_FALSY: TypeFlags = TypeFlags::from_bits_truncate(
    TypeFlags::StringLiteral.bits()
        | TypeFlags::NumberLiteral.bits()
        | TypeFlags::BigIntLiteral.bits()
        | TypeFlags::BooleanLiteral.bits()
        | TypeFlags::Void.bits()
        | TypeFlags::Undefined.bits()
        | TypeFlags::Null.bits(),
);
pub const TYPE_FLAGS_POSSIBLY_FALSY: TypeFlags = TypeFlags::from_bits_truncate(
    TYPE_FLAGS_DEFINITELY_FALSY.bits()
        | TypeFlags::String.bits()
        | TypeFlags::Number.bits()
        | TypeFlags::BigInt.bits()
        | TypeFlags::Boolean.bits(),
);
pub const TYPE_FLAGS_INTRINSIC: TypeFlags = TypeFlags::from_bits_truncate(
    TypeFlags::Any.bits()
        | TypeFlags::Unknown.bits()
        | TypeFlags::String.bits()
        | TypeFlags::Number.bits()
        | TypeFlags::BigInt.bits()
        | TypeFlags::ESSymbol.bits()
        | TypeFlags::Void.bits()
        | TypeFlags::Undefined.bits()
        | TypeFlags::Null.bits()
        | TypeFlags::Never.bits()
        | TypeFlags::NonPrimitive.bits(),
);
pub const TYPE_FLAGS_STRING_LIKE: TypeFlags = TypeFlags::from_bits_truncate(
    TypeFlags::String.bits()
        | TypeFlags::StringLiteral.bits()
        | TypeFlags::TemplateLiteral.bits()
        | TypeFlags::StringMapping.bits(),
);
pub const TYPE_FLAGS_NUMBER_LIKE: TypeFlags = TypeFlags::from_bits_truncate(
    TypeFlags::Number.bits() | TypeFlags::NumberLiteral.bits() | TypeFlags::Enum.bits(),
);
pub const TYPE_FLAGS_BIG_INT_LIKE: TypeFlags =
    TypeFlags::from_bits_truncate(TypeFlags::BigInt.bits() | TypeFlags::BigIntLiteral.bits());
pub const TYPE_FLAGS_BOOLEAN_LIKE: TypeFlags =
    TypeFlags::from_bits_truncate(TypeFlags::Boolean.bits() | TypeFlags::BooleanLiteral.bits());
pub const TYPE_FLAGS_ENUM_LIKE: TypeFlags =
    TypeFlags::from_bits_truncate(TypeFlags::Enum.bits() | TypeFlags::EnumLiteral.bits());
pub const TYPE_FLAGS_ES_SYMBOL_LIKE: TypeFlags =
    TypeFlags::from_bits_truncate(TypeFlags::ESSymbol.bits() | TypeFlags::UniqueESSymbol.bits());
pub const TYPE_FLAGS_VOID_LIKE: TypeFlags =
    TypeFlags::from_bits_truncate(TypeFlags::Void.bits() | TypeFlags::Undefined.bits());
pub const TYPE_FLAGS_PRIMITIVE: TypeFlags = TypeFlags::from_bits_truncate(
    TYPE_FLAGS_STRING_LIKE.bits()
        | TYPE_FLAGS_NUMBER_LIKE.bits()
        | TYPE_FLAGS_BIG_INT_LIKE.bits()
        | TYPE_FLAGS_BOOLEAN_LIKE.bits()
        | TYPE_FLAGS_ENUM_LIKE.bits()
        | TYPE_FLAGS_ES_SYMBOL_LIKE.bits()
        | TYPE_FLAGS_VOID_LIKE.bits()
        | TypeFlags::Null.bits(),
);
pub const TYPE_FLAGS_DEFINITELY_NON_NULLABLE: TypeFlags = TypeFlags::from_bits_truncate(
    TYPE_FLAGS_STRING_LIKE.bits()
        | TYPE_FLAGS_NUMBER_LIKE.bits()
        | TYPE_FLAGS_BIG_INT_LIKE.bits()
        | TYPE_FLAGS_BOOLEAN_LIKE.bits()
        | TYPE_FLAGS_ENUM_LIKE.bits()
        | TYPE_FLAGS_ES_SYMBOL_LIKE.bits()
        | TypeFlags::Object.bits()
        | TypeFlags::NonPrimitive.bits(),
);
pub const TYPE_FLAGS_DISJOINT_DOMAINS: TypeFlags = TypeFlags::from_bits_truncate(
    TypeFlags::NonPrimitive.bits()
        | TYPE_FLAGS_STRING_LIKE.bits()
        | TYPE_FLAGS_NUMBER_LIKE.bits()
        | TYPE_FLAGS_BIG_INT_LIKE.bits()
        | TYPE_FLAGS_BOOLEAN_LIKE.bits()
        | TYPE_FLAGS_ES_SYMBOL_LIKE.bits()
        | TYPE_FLAGS_VOID_LIKE.bits()
        | TypeFlags::Null.bits(),
);
pub const TYPE_FLAGS_UNION_OR_INTERSECTION: TypeFlags =
    TypeFlags::from_bits_truncate(TypeFlags::Union.bits() | TypeFlags::Intersection.bits());
pub const TYPE_FLAGS_STRUCTURED_TYPE: TypeFlags = TypeFlags::from_bits_truncate(
    TypeFlags::Object.bits() | TypeFlags::Union.bits() | TypeFlags::Intersection.bits(),
);
pub const TYPE_FLAGS_TYPE_VARIABLE: TypeFlags = TypeFlags::from_bits_truncate(
    TypeFlags::TypeParameter.bits() | TypeFlags::IndexedAccess.bits(),
);
