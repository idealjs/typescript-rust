//! Core type system definitions for the checker.
//!
//! Ported from `internal/checker/types.go`. The Go implementation uses a
//! `TypeData` interface with embedded structs (TypeBase → ConstrainedType →
//! StructuredType → ObjectType → TypeReference → InterfaceType → TupleType).
//! In Rust we flatten the hierarchy into a `TypeData` enum with one variant
//! per concrete type kind, and use pattern matching instead of interface
//! downcasts.

use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

use bitflags::bitflags;

use crate::ast::{Node, Symbol, SymbolFlags, SymbolTable};
use crate::core::tristate::Tristate;
use crate::evaluator;
use crate::jsnum;

// ────────────────────────────────────────────────────────────────────────────
// IDs
// ────────────────────────────────────────────────────────────────────────────

pub type TypeId = u32;
pub type SignatureId = u32;

// ────────────────────────────────────────────────────────────────────────────
// ParseFlags
// ────────────────────────────────────────────────────────────────────────────

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

// ────────────────────────────────────────────────────────────────────────────
// SignatureKind
// ────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SignatureKind {
    #[default]
    Call,
    Construct,
}

// ────────────────────────────────────────────────────────────────────────────
// ContextFlags
// ────────────────────────────────────────────────────────────────────────────

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

// ────────────────────────────────────────────────────────────────────────────
// TypeFormatFlags
// ────────────────────────────────────────────────────────────────────────────

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

// ────────────────────────────────────────────────────────────────────────────
// SymbolFormatFlags
// ────────────────────────────────────────────────────────────────────────────

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

// ────────────────────────────────────────────────────────────────────────────
// ExternalEmitHelpers
// ────────────────────────────────────────────────────────────────────────────

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

// ────────────────────────────────────────────────────────────────────────────
// TypeFlags
// ────────────────────────────────────────────────────────────────────────────

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

// Composite TypeFlags constants
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
pub const TYPE_FLAGS_INSTANTIABLE_NON_PRIMITIVE: TypeFlags = TypeFlags::from_bits_truncate(
    TYPE_FLAGS_TYPE_VARIABLE.bits()
        | TypeFlags::Conditional.bits()
        | TypeFlags::Substitution.bits(),
);
pub const TYPE_FLAGS_INSTANTIABLE_PRIMITIVE: TypeFlags = TypeFlags::from_bits_truncate(
    TypeFlags::Index.bits() | TypeFlags::TemplateLiteral.bits() | TypeFlags::StringMapping.bits(),
);
pub const TYPE_FLAGS_INSTANTIABLE: TypeFlags = TypeFlags::from_bits_truncate(
    TYPE_FLAGS_INSTANTIABLE_NON_PRIMITIVE.bits() | TYPE_FLAGS_INSTANTIABLE_PRIMITIVE.bits(),
);
pub const TYPE_FLAGS_STRUCTURED_OR_INSTANTIABLE: TypeFlags = TypeFlags::from_bits_truncate(
    TYPE_FLAGS_STRUCTURED_TYPE.bits() | TYPE_FLAGS_INSTANTIABLE.bits(),
);
pub const TYPE_FLAGS_OBJECT_FLAGS_TYPE: TypeFlags = TypeFlags::from_bits_truncate(
    TypeFlags::Any.bits()
        | TYPE_FLAGS_NULLABLE.bits()
        | TypeFlags::Never.bits()
        | TypeFlags::Object.bits()
        | TypeFlags::Union.bits()
        | TypeFlags::Intersection.bits(),
);
pub const TYPE_FLAGS_SIMPLIFIABLE: TypeFlags = TypeFlags::from_bits_truncate(
    TypeFlags::IndexedAccess.bits() | TypeFlags::Conditional.bits() | TypeFlags::Index.bits(),
);
pub const TYPE_FLAGS_SINGLETON: TypeFlags = TypeFlags::from_bits_truncate(
    TypeFlags::Any.bits()
        | TypeFlags::Unknown.bits()
        | TypeFlags::String.bits()
        | TypeFlags::Number.bits()
        | TypeFlags::Boolean.bits()
        | TypeFlags::BigInt.bits()
        | TypeFlags::ESSymbol.bits()
        | TypeFlags::Void.bits()
        | TypeFlags::Undefined.bits()
        | TypeFlags::Null.bits()
        | TypeFlags::Never.bits()
        | TypeFlags::NonPrimitive.bits(),
);
pub const TYPE_FLAGS_NARROWABLE: TypeFlags = TypeFlags::from_bits_truncate(
    TypeFlags::Any.bits()
        | TypeFlags::Unknown.bits()
        | TYPE_FLAGS_STRUCTURED_OR_INSTANTIABLE.bits()
        | TYPE_FLAGS_STRING_LIKE.bits()
        | TYPE_FLAGS_NUMBER_LIKE.bits()
        | TYPE_FLAGS_BIG_INT_LIKE.bits()
        | TYPE_FLAGS_BOOLEAN_LIKE.bits()
        | TypeFlags::ESSymbol.bits()
        | TypeFlags::UniqueESSymbol.bits()
        | TypeFlags::NonPrimitive.bits(),
);

// ────────────────────────────────────────────────────────────────────────────
// ObjectFlags
// ────────────────────────────────────────────────────────────────────────────

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct ObjectFlags: u32 {
        const None                                       = 0;
        const Class                                      = 1 << 0;
        const Interface                                  = 1 << 1;
        const Reference                                  = 1 << 2;
        const Tuple                                      = 1 << 3;
        const Anonymous                                  = 1 << 4;
        const Mapped                                     = 1 << 5;
        const Instantiated                               = 1 << 6;
        const ObjectLiteral                              = 1 << 7;
        const EvolvingArray                              = 1 << 8;
        const ObjectLiteralPatternWithComputedProperties = 1 << 9;
        const ReverseMapped                              = 1 << 10;
        const JsxAttributes                              = 1 << 11;
        const JSLiteral                                  = 1 << 12;
        const FreshLiteral                               = 1 << 13;
        const ArrayLiteral                               = 1 << 14;
        const PrimitiveUnion                             = 1 << 15;
        const ContainsWideningType                       = 1 << 16;
        const ContainsObjectOrArrayLiteral               = 1 << 17;
        const NonInferrableType                          = 1 << 18;
        const CouldContainTypeVariablesComputed          = 1 << 19;
        const CouldContainTypeVariables                  = 1 << 20;
        const MembersResolved                            = 1 << 21;
        const ContainsSpread                             = 1 << 22;
        const ObjectRestType                             = 1 << 23;
        const InstantiationExpressionType                = 1 << 24;
        const SingleSignatureType                        = 1 << 25;
        const IsClassInstanceClone                       = 1 << 26;
        const IdenticalBaseTypeCalculated                = 1 << 27;
        const IdenticalBaseTypeExists                    = 1 << 28;
        const UnresolvedMembers                          = 1 << 29;
        const FromTypeNode                                = 1 << 30;
        const IsGenericTypeComputed                      = 1 << 22; // reuse for union/intersection
        const IsGenericObjectType                        = 1 << 23;
        const IsGenericIndexType                         = 1 << 24;
        const ContainsIntersections                      = 1 << 25;
        const IsUnknownLikeUnionComputed                 = 1 << 26;
        const IsUnknownLikeUnion                         = 1 << 27;
        const IsNeverIntersectionComputed                = 1 << 25;
        const IsNeverIntersection                        = 1 << 26;
        const IsConstrainedTypeVariable                  = 1 << 27;
    }
}

pub const OBJECT_FLAGS_CLASS_OR_INTERFACE: ObjectFlags =
    ObjectFlags::from_bits_truncate(ObjectFlags::Class.bits() | ObjectFlags::Interface.bits());
pub const OBJECT_FLAGS_REQUIRES_WIDENING: ObjectFlags = ObjectFlags::from_bits_truncate(
    ObjectFlags::ContainsWideningType.bits() | ObjectFlags::ContainsObjectOrArrayLiteral.bits(),
);
pub const OBJECT_FLAGS_PROPAGATING_FLAGS: ObjectFlags = ObjectFlags::from_bits_truncate(
    ObjectFlags::ContainsWideningType.bits()
        | ObjectFlags::ContainsObjectOrArrayLiteral.bits()
        | ObjectFlags::NonInferrableType.bits(),
);
pub const OBJECT_FLAGS_INSTANTIATED_MAPPED: ObjectFlags =
    ObjectFlags::from_bits_truncate(ObjectFlags::Mapped.bits() | ObjectFlags::Instantiated.bits());
pub const OBJECT_FLAGS_IS_GENERIC_TYPE: ObjectFlags = ObjectFlags::from_bits_truncate(
    ObjectFlags::IsGenericObjectType.bits() | ObjectFlags::IsGenericIndexType.bits(),
);

// ────────────────────────────────────────────────────────────────────────────
// VarianceFlags
// ────────────────────────────────────────────────────────────────────────────

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct VarianceFlags: u32 {
        const None           = 0;
        const Covariant      = 1 << 0;
        const Contravariant  = 1 << 1;
        const Independent    = 1 << 2;
        const Unmeasurable   = 1 << 3;
        const Unreliable     = 1 << 4;
    }
}

pub const VARIANCE_FLAGS_BIVARIANT: VarianceFlags = VarianceFlags::from_bits_truncate(
    VarianceFlags::Covariant.bits() | VarianceFlags::Contravariant.bits(),
);
pub const VARIANCE_FLAGS_INVARIANT: VarianceFlags = VarianceFlags::None;
pub const VARIANCE_FLAGS_VARIANCE_MASK: VarianceFlags = VarianceFlags::from_bits_truncate(
    VarianceFlags::None.bits()
        | VarianceFlags::Covariant.bits()
        | VarianceFlags::Contravariant.bits()
        | VarianceFlags::Independent.bits(),
);
pub const VARIANCE_FLAGS_ALLOWS_STRUCTURAL_FALLBACK: VarianceFlags =
    VarianceFlags::from_bits_truncate(
        VarianceFlags::Unmeasurable.bits() | VarianceFlags::Unreliable.bits(),
    );

// ────────────────────────────────────────────────────────────────────────────
// AccessFlags
// ────────────────────────────────────────────────────────────────────────────

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct AccessFlags: u32 {
        const None                       = 0;
        const IncludeUndefined           = 1 << 0;
        const NoIndexSignatures          = 1 << 1;
        const Writing                    = 1 << 2;
        const CacheSymbol                = 1 << 3;
        const AllowMissing               = 1 << 4;
        const ExpressionPosition         = 1 << 5;
        const ReportDeprecated           = 1 << 6;
        const SuppressNoImplicitAnyError = 1 << 7;
        const Contextual                 = 1 << 8;
    }
}

pub const ACCESS_FLAGS_PERSISTENT: AccessFlags = AccessFlags::IncludeUndefined;

// ────────────────────────────────────────────────────────────────────────────
// NodeCheckFlags
// ────────────────────────────────────────────────────────────────────────────

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct NodeCheckFlags: u32 {
        const None                                     = 0;
        const TypeChecked                              = 1 << 0;
        const ContextChecked                           = 1 << 6;
        const EnumValuesComputed                       = 1 << 10;
        const AssignmentsMarked                        = 1 << 17;
        const ContainsClassWithPrivateIdentifiers      = 1 << 20;
        const ContainsSuperPropertyInStaticInitializer = 1 << 21;
        const InCheckIdentifier                        = 1 << 22;
        const InitializerIsUndefined                   = 1 << 24;
        const InitializerIsUndefinedComputed           = 1 << 25;
    }
}

// ────────────────────────────────────────────────────────────────────────────
// ElementFlags (for tuple types)
// ────────────────────────────────────────────────────────────────────────────

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct ElementFlags: u32 {
        const None     = 0;
        const Required = 1 << 0;
        const Optional = 1 << 1;
        const Rest     = 1 << 2;
        const Variadic = 1 << 3;
    }
}

pub const ELEMENT_FLAGS_FIXED: ElementFlags =
    ElementFlags::from_bits_truncate(ElementFlags::Required.bits() | ElementFlags::Optional.bits());
pub const ELEMENT_FLAGS_VARIABLE: ElementFlags =
    ElementFlags::from_bits_truncate(ElementFlags::Rest.bits() | ElementFlags::Variadic.bits());
pub const ELEMENT_FLAGS_NON_REQUIRED: ElementFlags = ElementFlags::from_bits_truncate(
    ElementFlags::Optional.bits() | ElementFlags::Rest.bits() | ElementFlags::Variadic.bits(),
);
pub const ELEMENT_FLAGS_NON_REST: ElementFlags = ElementFlags::from_bits_truncate(
    ElementFlags::Required.bits() | ElementFlags::Optional.bits() | ElementFlags::Variadic.bits(),
);

// ────────────────────────────────────────────────────────────────────────────
// SignatureFlags
// ────────────────────────────────────────────────────────────────────────────

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct SignatureFlags: u32 {
        const None                                     = 0;
        const HasRestParameter                         = 1 << 0;
        const HasLiteralTypes                          = 1 << 1;
        const Construct                                = 1 << 2;
        const Abstract                                 = 1 << 3;
        const IsInnerCallChain                         = 1 << 4;
        const IsOuterCallChain                         = 1 << 5;
        const IsUntypedSignatureInJSFile               = 1 << 6;
        const IsNonInferrable                          = 1 << 7;
        const IsSignatureCandidateForOverloadFailure   = 1 << 8;
    }
}

pub const SIGNATURE_FLAGS_PROPAGATING_FLAGS: SignatureFlags = SignatureFlags::from_bits_truncate(
    SignatureFlags::HasRestParameter.bits()
        | SignatureFlags::HasLiteralTypes.bits()
        | SignatureFlags::Construct.bits()
        | SignatureFlags::Abstract.bits()
        | SignatureFlags::IsUntypedSignatureInJSFile.bits()
        | SignatureFlags::IsSignatureCandidateForOverloadFailure.bits(),
);
pub const SIGNATURE_FLAGS_CALL_CHAIN_FLAGS: SignatureFlags = SignatureFlags::from_bits_truncate(
    SignatureFlags::IsInnerCallChain.bits() | SignatureFlags::IsOuterCallChain.bits(),
);

// ────────────────────────────────────────────────────────────────────────────
// IndexFlags
// ────────────────────────────────────────────────────────────────────────────

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct IndexFlags: u32 {
        const None              = 0;
        const StringsOnly       = 1 << 0;
        const NoIndexSignatures = 1 << 1;
        const NoReducibleCheck  = 1 << 2;
    }
}

// ────────────────────────────────────────────────────────────────────────────
// Ternary (relation comparison result)
// ────────────────────────────────────────────────────────────────────────────

/// Three-valued relation result used in type comparison.
///
/// `False < Unknown < Maybe < True` for `&` (min) and `|` (max).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i8)]
pub enum Ternary {
    False = 0,
    Unknown = 1,
    Maybe = 3,
    True = -1,
}

impl Ternary {
    /// Semantic rank: False(0) < Unknown(1) < Maybe(2) < True(3).
    fn rank(self) -> u8 {
        match self {
            Ternary::False => 0,
            Ternary::Unknown => 1,
            Ternary::Maybe => 2,
            Ternary::True => 3,
        }
    }

    pub fn and(self, other: Ternary) -> Ternary {
        if self.rank() <= other.rank() {
            self
        } else {
            other
        }
    }

    pub fn or(self, other: Ternary) -> Ternary {
        if self.rank() >= other.rank() {
            self
        } else {
            other
        }
    }

    pub fn not(self) -> Ternary {
        match self {
            Ternary::True => Ternary::False,
            Ternary::False => Ternary::True,
            Ternary::Unknown => Ternary::Unknown,
            Ternary::Maybe => Ternary::Maybe,
        }
    }

    pub fn is_true(self) -> bool {
        self == Ternary::True
    }
    pub fn is_false(self) -> bool {
        self == Ternary::False
    }
    pub fn is_maybe(self) -> bool {
        self == Ternary::Maybe
    }
    pub fn is_unknown(self) -> bool {
        self == Ternary::Unknown
    }
}

impl PartialOrd for Ternary {
    fn partial_cmp(&self, other: &Ternary) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Ternary {
    fn cmp(&self, other: &Ternary) -> std::cmp::Ordering {
        self.rank().cmp(&other.rank())
    }
}

impl From<bool> for Ternary {
    fn from(b: bool) -> Self {
        if b { Ternary::True } else { Ternary::False }
    }
}

impl std::ops::BitAnd for Ternary {
    type Output = Ternary;
    fn bitand(self, other: Ternary) -> Ternary {
        self.and(other)
    }
}

impl std::ops::BitOr for Ternary {
    type Output = Ternary;
    fn bitor(self, other: Ternary) -> Ternary {
        self.or(other)
    }
}

impl std::ops::Not for Ternary {
    type Output = Ternary;
    fn not(self) -> Ternary {
        Ternary::not(self)
    }
}

pub type TypeComparer = fn(&Type, &Type, bool) -> Ternary;

// ────────────────────────────────────────────────────────────────────────────
// TypeAlias
// ────────────────────────────────────────────────────────────────────────────

/// A type alias reference attached to a `Type`.
#[derive(Debug)]
pub struct TypeAlias {
    pub symbol: Option<Arc<Symbol>>,
    pub type_arguments: Vec<Arc<Type>>,
}

impl TypeAlias {
    pub fn new(symbol: Option<Arc<Symbol>>, type_arguments: Vec<Arc<Type>>) -> Self {
        Self {
            symbol,
            type_arguments,
        }
    }
}

// ────────────────────────────────────────────────────────────────────────────
// TypeMapper (forward declaration; full impl in mapper.rs)
// ────────────────────────────────────────────────────────────────────────────

/// Maps one type to another (e.g., type parameter → inferred type).
///
/// Takes `&Arc<Type>` (not `&Type`) so that mappers can return the input
/// type unchanged via `Arc::clone` when no mapping applies — this matches
/// Go's `Mapper.map` returning the input interface value directly and
/// avoids the prior placeholder behavior of returning an unrelated target.
pub type MapFn = Arc<dyn Fn(&Arc<Type>) -> Arc<Type> + Send + Sync>;

pub struct TypeMapper {
    pub kind: TypeMapperKind,
    pub map_fn: MapFn,
    pub maps_this_only: bool,
}

impl std::fmt::Debug for TypeMapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TypeMapper")
            .field("kind", &self.kind)
            .field("maps_this_only", &self.maps_this_only)
            .finish()
    }
}

impl Clone for TypeMapper {
    fn clone(&self) -> Self {
        Self {
            kind: self.kind,
            map_fn: Arc::clone(&self.map_fn),
            maps_this_only: self.maps_this_only,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TypeMapperKind {
    #[default]
    Unknown,
    Simple,
    Array,
    Merged,
}

impl TypeMapper {
    pub fn new(map_fn: MapFn, kind: TypeMapperKind, maps_this_only: bool) -> Self {
        Self {
            kind,
            map_fn,
            maps_this_only,
        }
    }

    pub fn map(&self, t: &Arc<Type>) -> Arc<Type> {
        (self.map_fn)(t)
    }

    pub fn maps_this_only(&self) -> bool {
        self.maps_this_only
    }
}

// ────────────────────────────────────────────────────────────────────────────
// Type and TypeData
// ────────────────────────────────────────────────────────────────────────────

/// The core type representation. Equivalent to `*checker.Type` in Go.
///
/// Types are always shared via `Arc<Type>`. The `data` field is an enum
/// that discriminates the kind of type (intrinsic, literal, object, union,
/// etc.), replacing Go's `TypeData` interface and embedded-struct hierarchy.
#[derive(Debug)]
pub struct Type {
    pub flags: TypeFlags,
    pub object_flags: ObjectFlags,
    pub id: TypeId,
    pub symbol: Option<Arc<Symbol>>,
    pub alias: Option<Box<TypeAlias>>,
    pub data: TypeData,
}

/// Type-specific data, discriminated by enum variant.
///
/// In Go, this is the `TypeData` interface with implementations:
/// `IntrinsicType`, `LiteralType`, `ObjectType`, `InterfaceType`, `TupleType`,
/// `UnionType`, `IntersectionType`, `TypeParameter`, `IndexType`,
/// `IndexedAccessType`, `TemplateLiteralType`, `StringMappingType`,
/// `SubstitutionType`, `ConditionalType`, etc.
#[derive(Debug)]
pub enum TypeData {
    Intrinsic(IntrinsicTypeData),
    Literal(LiteralTypeData),
    UniqueESSymbol(UniqueESSymbolTypeData),
    Object(ObjectTypeData),
    Interface(InterfaceTypeData),
    Tuple(TupleTypeData),
    Mapped(MappedTypeData),
    ReverseMapped(ReverseMappedTypeData),
    EvolvingArray(EvolvingArrayTypeData),
    InstantiationExpression(InstantiationExpressionTypeData),
    Union(UnionTypeData),
    Intersection(IntersectionTypeData),
    TypeParameter(TypeParameterData),
    Index(IndexTypeData),
    IndexedAccess(IndexedAccessTypeData),
    TemplateLiteral(TemplateLiteralTypeData),
    StringMapping(StringMappingTypeData),
    Substitution(SubstitutionTypeData),
    Conditional(ConditionalTypeData),
}

/// Data common to constrained types (types with a computed base constraint).
#[derive(Debug, Default)]
pub struct ConstrainedTypeData {
    pub resolved_base_constraint: OnceLock<Arc<Type>>,
}

/// Data common to structured types (types with members, properties, signatures).
#[derive(Debug, Default)]
pub struct StructuredTypeData {
    pub constrained: ConstrainedTypeData,
    pub members: SymbolTable,
    pub properties: Vec<Arc<Symbol>>,
    pub signatures: Vec<Arc<Signature>>,
    pub call_signature_count: usize,
    pub index_infos: Vec<Arc<IndexInfo>>,
    pub object_type_without_abstract_construct_signatures: OnceLock<Arc<Type>>,
}

impl StructuredTypeData {
    pub fn call_signatures(&self) -> &[Arc<Signature>] {
        &self.signatures[..self.call_signature_count]
    }

    pub fn construct_signatures(&self) -> &[Arc<Signature>] {
        &self.signatures[self.call_signature_count..]
    }
}

/// Data common to object types (target + mapper for instantiated types).
#[derive(Debug, Default)]
pub struct ObjectTypeData {
    pub structured: StructuredTypeData,
    pub target: Option<Arc<Type>>,
    pub mapper: Option<Arc<TypeMapper>>,
    /// Type arguments for type references (e.g., `T` in `Array<T>`).
    pub type_arguments: Vec<Arc<Type>>,
}

// ────────────────────────────────────────────────────────────────────────────
// Concrete TypeData variants
// ────────────────────────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct IntrinsicTypeData {
    pub intrinsic_name: String,
}

#[derive(Debug)]
pub struct LiteralTypeData {
    pub value: LiteralValue,
    pub fresh_type: OnceLock<Arc<Type>>,
    pub regular_type: OnceLock<Arc<Type>>,
}

/// A literal type value (string, number, boolean, bigint, or computed enum).
#[derive(Debug, Clone, PartialEq)]
pub enum LiteralValue {
    String(String),
    Number(jsnum::Number),
    BigInt(jsnum::PseudoBigInt),
    Boolean(bool),
    /// Computed enum member (value not yet known).
    None,
}

impl LiteralValue {
    pub fn to_string(&self) -> String {
        match self {
            LiteralValue::String(s) => format!("\"{}\"", s),
            LiteralValue::Number(n) => n.to_string(),
            LiteralValue::BigInt(b) => b.to_string(),
            LiteralValue::Boolean(b) => b.to_string(),
            LiteralValue::None => String::new(),
        }
    }
}

#[derive(Debug)]
pub struct UniqueESSymbolTypeData {
    pub name: String,
}

#[derive(Debug, Default)]
pub struct InterfaceTypeData {
    pub object: ObjectTypeData,
    pub all_type_parameters: Vec<Arc<Type>>,
    pub outer_type_parameter_count: usize,
    pub this_type: Option<Arc<Type>>,
    pub base_types_resolved: bool,
    pub declared_members_resolved: bool,
    pub resolved_base_constructor_type: OnceLock<Arc<Type>>,
    pub resolved_base_types: Vec<Arc<Type>>,
    pub declared_members: SymbolTable,
    pub declared_call_signatures: Vec<Arc<Signature>>,
    pub declared_construct_signatures: Vec<Arc<Signature>>,
    pub declared_index_infos: Vec<Arc<IndexInfo>>,
}

impl InterfaceTypeData {
    pub fn outer_type_parameters(&self) -> &[Arc<Type>] {
        if self.all_type_parameters.is_empty() {
            return &[];
        }
        &self.all_type_parameters[..self.outer_type_parameter_count]
    }

    pub fn local_type_parameters(&self) -> &[Arc<Type>] {
        if self.all_type_parameters.is_empty() {
            return &[];
        }
        let end = self.all_type_parameters.len().saturating_sub(1);
        &self.all_type_parameters[self.outer_type_parameter_count..end]
    }

    pub fn type_parameters(&self) -> &[Arc<Type>] {
        if self.all_type_parameters.is_empty() {
            return &[];
        }
        let end = self.all_type_parameters.len() - 1;
        &self.all_type_parameters[..end]
    }
}

#[derive(Debug, Default)]
pub struct TupleTypeData {
    pub interface_data: InterfaceTypeData,
    pub element_infos: Vec<TupleElementInfo>,
    pub min_length: usize,
    pub fixed_length: usize,
    pub combined_flags: ElementFlags,
    pub readonly: bool,
}

#[derive(Debug, Clone)]
pub struct TupleElementInfo {
    pub flags: ElementFlags,
    pub labeled_declaration: Option<Arc<Node>>,
    /// The element type. Stored alongside `flags` so that the relater can
    /// compare tuple element types without re-resolving the structured
    /// member symbols. Mirrors `TupleElementInfo.Type` in Go.
    pub type_: Option<Arc<Type>>,
}

#[derive(Debug)]
pub struct MappedTypeData {
    pub object: ObjectTypeData,
    pub declaration: Option<Arc<Node>>,
    pub type_parameter: Option<Arc<Type>>,
    pub constraint_type: Option<Arc<Type>>,
    pub name_type: Option<Arc<Type>>,
    pub template_type: Option<Arc<Type>>,
    pub modifiers_type: Option<Arc<Type>>,
    pub resolved_apparent_type: OnceLock<Arc<Type>>,
    pub contains_error: bool,
}

#[derive(Debug)]
pub struct ReverseMappedTypeData {
    pub object: ObjectTypeData,
    pub source: Option<Arc<Type>>,
    pub mapped_type: Option<Arc<Type>>,
    pub constraint_type: Option<Arc<Type>>,
}

#[derive(Debug)]
pub struct EvolvingArrayTypeData {
    pub object: ObjectTypeData,
    pub element_type: Option<Arc<Type>>,
    pub final_array_type: OnceLock<Arc<Type>>,
}

#[derive(Debug)]
pub struct InstantiationExpressionTypeData {
    pub object: ObjectTypeData,
    pub node: Option<Arc<Node>>,
}

#[derive(Debug, Default)]
pub struct UnionOrIntersectionTypeData {
    pub structured: StructuredTypeData,
    pub types: Vec<Arc<Type>>,
}

#[derive(Debug, Default)]
pub struct UnionTypeData {
    pub union_or_intersection: UnionOrIntersectionTypeData,
    pub resolved_reduced_type: OnceLock<Arc<Type>>,
    pub regular_type: OnceLock<Arc<Type>>,
    pub origin: Option<Arc<Type>>,
    pub key_property_name: Option<String>,
    pub constituent_map: HashMap<TypeId, Arc<Type>>,
}

#[derive(Debug, Default)]
pub struct IntersectionTypeData {
    pub union_or_intersection: UnionOrIntersectionTypeData,
    pub resolved_apparent_type: OnceLock<Arc<Type>>,
    pub unique_literal_filled_instantiation: OnceLock<Arc<Type>>,
}

#[derive(Debug)]
pub struct TypeParameterData {
    pub constrained: ConstrainedTypeData,
    pub constraint: Option<Arc<Type>>,
    pub target: Option<Arc<Type>>,
    pub mapper: Option<Arc<TypeMapper>>,
    pub is_this_type: bool,
    pub resolved_default_type: OnceLock<Arc<Type>>,
}

#[derive(Debug)]
pub struct IndexTypeData {
    pub constrained: ConstrainedTypeData,
    pub target: Option<Arc<Type>>,
    pub index_flags: IndexFlags,
}

#[derive(Debug)]
pub struct IndexedAccessTypeData {
    pub constrained: ConstrainedTypeData,
    pub object_type: Option<Arc<Type>>,
    pub index_type: Option<Arc<Type>>,
    pub access_flags: AccessFlags,
}

#[derive(Debug)]
pub struct TemplateLiteralTypeData {
    pub constrained: ConstrainedTypeData,
    pub texts: Vec<String>,
    pub types: Vec<Arc<Type>>,
}

#[derive(Debug)]
pub struct StringMappingTypeData {
    pub constrained: ConstrainedTypeData,
    pub target: Option<Arc<Type>>,
}

#[derive(Debug)]
pub struct SubstitutionTypeData {
    pub constrained: ConstrainedTypeData,
    pub base_type: Option<Arc<Type>>,
    pub constraint: Option<Arc<Type>>,
}

#[derive(Debug)]
pub struct ConditionalRoot {
    pub node: Option<Arc<Node>>,
    pub check_type: Option<Arc<Type>>,
    pub extends_type: Option<Arc<Type>>,
    pub is_distributive: bool,
    /// When the root is distributive (the check type as written is a naked
    /// type parameter), the symbol of that type parameter. Distribution
    /// substitutes this symbol per union constituent while re-resolving the
    /// extends/branch nodes (Go: `prependTypeMapping(checkType, t,
    /// newMapper)` in `getConditionalTypeInstantiation`).
    pub check_type_parameter_symbol: Option<Arc<crate::ast::Symbol>>,
    pub infer_type_parameters: Vec<Arc<Type>>,
    pub outer_type_parameters: Vec<Arc<Type>>,
    pub alias: Option<Box<TypeAlias>>,
    /// Clone of the checker's scope-stack CONTAINER IDS captured when this
    /// root was built. Deferred conditionals created mid-alias-instantiation
    /// live lexically inside the alias declaration; when their branches are
    /// finally resolved from a distant context (call-checking fallback
    /// instantiations), that lexical chain is gone and identifier resolution
    /// in the branch nodes would fail. Resolution temporarily re-pushes the
    /// non-common suffix of this chain (identifiers resolve by container id
    /// lookup in the symbol map; no node handles are needed).
    pub creation_scopes: Vec<u64>,
}

#[derive(Debug)]
pub struct ConditionalTypeData {
    pub constrained: ConstrainedTypeData,
    pub root: Option<Box<ConditionalRoot>>,
    pub check_type: Option<Arc<Type>>,
    pub extends_type: Option<Arc<Type>>,
    pub resolved_true_type: OnceLock<Arc<Type>>,
    pub resolved_false_type: OnceLock<Arc<Type>>,
    pub resolved_inferred_true_type: OnceLock<Arc<Type>>,
    pub resolved_default_constraint: OnceLock<Arc<Type>>,
    pub resolved_constraint_of_distributive: OnceLock<Arc<Type>>,
    pub mapper: Option<Arc<TypeMapper>>,
    pub combined_mapper: Option<Arc<TypeMapper>>,
    /// Snapshot of the checker's `type_argument_stack` at the moment this
    /// conditional INSTANCE was created (e.g. mid-alias-instantiation, while
    /// the alias's own type parameters shadowed their arguments), keyed by
    /// the raw symbol-pointer value (`&Symbol as *const _ as usize`) that
    /// `type_argument_stack` itself keys on. A generic alias body stays
    /// deferred at creation; when a *later* substitution makes its check
    /// type concrete and the branches must finally be resolved, those
    /// branch NODES still reference the alias-local type-parameter symbols
    /// — resolving them without this snapshot loses the bindings entirely
    /// and produces garbage (`keyof <unresolved>`). Go carries an
    /// equivalent `mapper` on every deferred conditional
    /// (`newConditionalType(root, mapper, combinedMapper)`).
    pub creation_type_argument_stack: Vec<HashMap<usize, Arc<Type>>>,
}

// ────────────────────────────────────────────────────────────────────────────
// Type accessors and helpers
// ────────────────────────────────────────────────────────────────────────────

impl Type {
    pub fn new(flags: TypeFlags, data: TypeData) -> Self {
        Self {
            flags,
            object_flags: ObjectFlags::empty(),
            id: 0,
            symbol: None,
            alias: None,
            data,
        }
    }

    pub fn is_union(&self) -> bool {
        self.flags.contains(TypeFlags::Union)
    }

    pub fn is_intersection(&self) -> bool {
        self.flags.contains(TypeFlags::Intersection)
    }

    pub fn is_string(&self) -> bool {
        self.flags.contains(TypeFlags::String)
    }

    pub fn is_string_literal(&self) -> bool {
        self.flags.contains(TypeFlags::StringLiteral)
    }

    pub fn is_number_literal(&self) -> bool {
        self.flags.contains(TypeFlags::NumberLiteral)
    }

    pub fn is_big_int_literal(&self) -> bool {
        self.flags.contains(TypeFlags::BigIntLiteral)
    }

    pub fn is_enum_literal(&self) -> bool {
        self.flags.contains(TypeFlags::EnumLiteral)
    }

    pub fn is_boolean_like(&self) -> bool {
        self.flags.intersects(TYPE_FLAGS_BOOLEAN_LIKE)
    }

    pub fn is_string_like(&self) -> bool {
        self.flags.intersects(TYPE_FLAGS_STRING_LIKE)
    }

    pub fn is_class(&self) -> bool {
        self.object_flags.contains(ObjectFlags::Class)
    }

    pub fn is_type_parameter(&self) -> bool {
        self.flags.contains(TypeFlags::TypeParameter)
    }

    pub fn is_index(&self) -> bool {
        self.flags.contains(TypeFlags::Index)
    }

    /// Returns the constituent types of a union, or `[self]` for non-unions.
    /// Returns empty for `never`.
    pub fn distributed(&self) -> Vec<Arc<Type>> {
        if self.flags.contains(TypeFlags::Union) {
            if let TypeData::Union(u) = &self.data {
                return u.union_or_intersection.types.clone();
            }
        }
        if self.flags.contains(TypeFlags::Never) {
            return Vec::new();
        }
        // Non-union types return themselves; caller must wrap in Arc
        Vec::new() // placeholder — real impl needs Arc<Self>
    }

    /// Get target type (for references, type parameters, index types, etc.)
    pub fn target(&self) -> Option<&Arc<Type>> {
        match &self.data {
            TypeData::Object(o) => o.target.as_ref(),
            TypeData::Interface(i) => i.object.target.as_ref(),
            TypeData::Tuple(t) => t.interface_data.object.target.as_ref(),
            TypeData::Mapped(m) => m.object.target.as_ref(),
            TypeData::ReverseMapped(r) => r.object.target.as_ref(),
            TypeData::EvolvingArray(e) => e.object.target.as_ref(),
            TypeData::InstantiationExpression(i) => i.object.target.as_ref(),
            TypeData::TypeParameter(t) => t.target.as_ref(),
            TypeData::Index(i) => i.target.as_ref(),
            TypeData::StringMapping(s) => s.target.as_ref(),
            _ => None,
        }
    }

    /// Get mapper (for instantiated types)
    pub fn mapper(&self) -> Option<&Arc<TypeMapper>> {
        match &self.data {
            TypeData::Object(o) => o.mapper.as_ref(),
            TypeData::Interface(i) => i.object.mapper.as_ref(),
            TypeData::Tuple(t) => t.interface_data.object.mapper.as_ref(),
            TypeData::Mapped(m) => m.object.mapper.as_ref(),
            TypeData::ReverseMapped(r) => r.object.mapper.as_ref(),
            TypeData::EvolvingArray(e) => e.object.mapper.as_ref(),
            TypeData::InstantiationExpression(i) => i.object.mapper.as_ref(),
            TypeData::TypeParameter(t) => t.mapper.as_ref(),
            TypeData::Conditional(c) => c.mapper.as_ref(),
            _ => None,
        }
    }

    /// Get constituent types (for union/intersection/template literal)
    pub fn types(&self) -> Option<&[Arc<Type>]> {
        match &self.data {
            TypeData::Union(u) => Some(&u.union_or_intersection.types),
            TypeData::Intersection(i) => Some(&i.union_or_intersection.types),
            TypeData::TemplateLiteral(t) => Some(&t.types),
            _ => None,
        }
    }

    /// Get structured type data if this is a structured type.
    pub fn as_structured(&self) -> Option<&StructuredTypeData> {
        match &self.data {
            TypeData::Object(o) => Some(&o.structured),
            TypeData::Interface(i) => Some(&i.object.structured),
            TypeData::Tuple(t) => Some(&t.interface_data.object.structured),
            TypeData::Mapped(m) => Some(&m.object.structured),
            TypeData::ReverseMapped(r) => Some(&r.object.structured),
            TypeData::EvolvingArray(e) => Some(&e.object.structured),
            TypeData::InstantiationExpression(i) => Some(&i.object.structured),
            TypeData::Union(u) => Some(&u.union_or_intersection.structured),
            TypeData::Intersection(i) => Some(&i.union_or_intersection.structured),
            _ => None,
        }
    }

    /// Get object type data if this is an object type.
    pub fn as_object(&self) -> Option<&ObjectTypeData> {
        match &self.data {
            TypeData::Object(o) => Some(o),
            TypeData::Interface(i) => Some(&i.object),
            TypeData::Tuple(t) => Some(&t.interface_data.object),
            TypeData::Mapped(m) => Some(&m.object),
            TypeData::ReverseMapped(r) => Some(&r.object),
            TypeData::EvolvingArray(e) => Some(&e.object),
            TypeData::InstantiationExpression(i) => Some(&i.object),
            _ => None,
        }
    }

    /// Get interface type data if this is an interface type.
    pub fn as_interface(&self) -> Option<&InterfaceTypeData> {
        match &self.data {
            TypeData::Interface(i) => Some(i),
            TypeData::Tuple(t) => Some(&t.interface_data),
            _ => None,
        }
    }

    /// Get union or intersection type data.
    pub fn as_union_or_intersection(&self) -> Option<&UnionOrIntersectionTypeData> {
        match &self.data {
            TypeData::Union(u) => Some(&u.union_or_intersection),
            TypeData::Intersection(i) => Some(&i.union_or_intersection),
            _ => None,
        }
    }

    /// Get intrinsic name (for intrinsic types).
    pub fn intrinsic_name(&self) -> Option<&str> {
        if let TypeData::Intrinsic(i) = &self.data {
            Some(&i.intrinsic_name)
        } else {
            None
        }
    }

    /// Get literal value (for literal types).
    pub fn literal_value(&self) -> Option<&LiteralValue> {
        if let TypeData::Literal(l) = &self.data {
            Some(&l.value)
        } else {
            None
        }
    }
}

// ────────────────────────────────────────────────────────────────────────────
// Signature
// ────────────────────────────────────────────────────────────────────────────

/// A function or constructor signature.
#[derive(Debug)]
pub struct Signature {
    pub id: SignatureId,
    pub flags: SignatureFlags,
    pub min_argument_count: i32,
    pub resolved_min_argument_count: i32,
    pub declaration: Option<Arc<Node>>,
    pub type_parameters: Vec<Arc<Type>>,
    pub parameters: Vec<Arc<Symbol>>,
    pub this_parameter: Option<Arc<Symbol>>,
    pub resolved_return_type: OnceLock<Arc<Type>>,
    pub resolved_type_predicate: Option<Box<TypePredicate>>,
    pub target: Option<Arc<Signature>>,
    pub mapper: Option<Arc<TypeMapper>>,
    pub isolated_signature_type: OnceLock<Arc<Type>>,
    /// Substituted parameter types for contextually instantiated
    /// signatures (Go: `getSignatureInstantiation` /
    /// `instantiateSignatureInContextOf`). Keyed by PARAMETER INDEX (the
    /// rest parameter keeps its array type with the element substituted).
    /// `None` resolves through the parameter symbols as usual.
    pub instantiated_parameter_types: Option<Vec<Arc<Type>>>,
}

impl Signature {
    pub fn new() -> Self {
        Self {
            id: 0,
            flags: SignatureFlags::None,
            min_argument_count: 0,
            resolved_min_argument_count: 0,
            declaration: None,
            type_parameters: Vec::new(),
            parameters: Vec::new(),
            this_parameter: None,
            resolved_return_type: OnceLock::new(),
            resolved_type_predicate: None,
            target: None,
            mapper: None,
            isolated_signature_type: OnceLock::new(),
            instantiated_parameter_types: None,
        }
    }

    pub fn has_rest_parameter(&self) -> bool {
        self.flags.contains(SignatureFlags::HasRestParameter)
    }

    pub fn min_argument_count(&self) -> usize {
        self.min_argument_count as usize
    }
}

impl Default for Signature {
    fn default() -> Self {
        Self::new()
    }
}

/// Composite signature (union or intersection of signatures).
#[derive(Debug)]
pub struct CompositeSignature {
    pub is_union: bool,
    pub signatures: Vec<Arc<Signature>>,
}

// ────────────────────────────────────────────────────────────────────────────
// TypePredicate
// ────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TypePredicateKind {
    #[default]
    This,
    Identifier,
    AssertsThis,
    AssertsIdentifier,
}

#[derive(Debug, Clone)]
pub struct TypePredicate {
    pub kind: TypePredicateKind,
    pub parameter_index: i32,
    pub parameter_name: String,
    pub t: Option<Arc<Type>>,
}

// ────────────────────────────────────────────────────────────────────────────
// IndexInfo
// ────────────────────────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct IndexInfo {
    pub key_type: Option<Arc<Type>>,
    pub value_type: Option<Arc<Type>>,
    pub is_readonly: bool,
    pub declaration: Option<Arc<Node>>,
    pub index_symbol: Option<Arc<Symbol>>,
    pub components: Vec<Arc<Node>>,
}

// ────────────────────────────────────────────────────────────────────────────
// Links types (side-table data for nodes and symbols)
// ────────────────────────────────────────────────────────────────────────────

/// Links for referenced symbols.
#[derive(Debug, Default)]
pub struct SymbolReferenceLinks {
    pub reference_kinds: crate::ast::SymbolFlags,
}

/// Links for value symbols.
#[derive(Debug, Default)]
pub struct ValueSymbolLinks {
    pub resolved_type: Option<Arc<Type>>,
    pub write_type: Option<Arc<Type>>,
    pub target: Option<Arc<Symbol>>,
    pub mapper: Option<Arc<TypeMapper>>,
    pub name_type: Option<Arc<Type>>,
    pub containing_type: Option<Arc<Type>>,
    pub function_or_constructor_checked: bool,
}

/// Links for mapped symbols.
#[derive(Debug, Default)]
pub struct MappedSymbolLinks {
    pub key_type: Option<Arc<Type>>,
    pub synthetic_origin: Option<Arc<Symbol>>,
}

/// Links for deferred type symbols.
#[derive(Debug, Default)]
pub struct DeferredSymbolLinks {
    pub parent: Option<Arc<Type>>,
    pub constituents: Vec<Arc<Type>>,
    pub write_constituents: Vec<Arc<Type>>,
}

/// Links for alias symbols.
#[derive(Debug, Default)]
pub struct AliasSymbolLinks {
    pub immediate_target: Option<Arc<Symbol>>,
    pub alias_target: Option<Arc<Symbol>>,
    pub referenced: bool,
    pub type_only_declaration: Option<Arc<Node>>,
}

/// Links for module symbols.
#[derive(Debug, Default)]
pub struct ModuleSymbolLinks {
    pub resolved_exports: SymbolTable,
    pub type_only_export_star_map: HashMap<String, Arc<Node>>,
    pub exports_checked: bool,
}

#[derive(Debug, Default)]
pub struct ReverseMappedSymbolLinks {
    pub property_type: Option<Arc<Type>>,
    pub mapped_type: Option<Arc<Type>>,
    pub constraint_type: Option<Arc<Type>>,
}

#[derive(Debug, Default)]
pub struct LateBoundLinks {
    pub late_symbol: Option<Arc<Symbol>>,
}

#[derive(Debug, Default)]
pub struct ExportTypeLinks {
    pub target: Option<Arc<Symbol>>,
    pub originating_import: Option<Arc<Node>>,
}

#[derive(Debug, Default)]
pub struct TypeAliasLinks {
    pub declared_type: Option<Arc<Type>>,
    pub type_parameters: Vec<Arc<Type>>,
    pub is_constructor_declared_property: bool,
}

#[derive(Debug, Default)]
pub struct DeclaredTypeLinks {
    pub declared_type: Option<Arc<Type>>,
    pub interface_checked: bool,
    pub index_signatures_checked: bool,
    pub type_parameters_checked: bool,
    pub enum_checked: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ExhaustiveState {
    #[default]
    Unknown,
    Computing,
    False,
    True,
}

#[derive(Debug, Default)]
pub struct SwitchStatementLinks {
    pub exhaustive_state: ExhaustiveState,
    pub switch_types_computed: bool,
    pub witnesses_computed: bool,
    pub switch_types: Vec<Arc<Type>>,
    pub witnesses: Vec<String>,
}

#[derive(Debug, Default)]
pub struct ArrayLiteralLinks {
    pub indices_computed: bool,
    pub first_spread_index: i32,
    pub last_spread_index: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MembersOrExportsResolutionKind {
    #[default]
    ResolvedExports,
    ResolvedMembers,
}

#[derive(Debug, Default)]
pub struct MembersAndExportsLinks {
    pub resolved_exports: SymbolTable,
    pub resolved_members: SymbolTable,
}

#[derive(Debug, Default)]
pub struct SpreadLinks {
    pub left_spread: Option<Arc<Symbol>>,
    pub right_spread: Option<Arc<Symbol>>,
}

#[derive(Debug, Default)]
pub struct VarianceLinks {
    pub variances: Vec<VarianceFlags>,
}

#[derive(Debug, Default)]
pub struct MarkedAssignmentSymbolLinks {
    pub last_assignment_pos: i32,
    pub has_definite_assignment: bool,
}

/// Node-level check links.
#[derive(Debug, Default)]
pub struct NodeLinks {
    pub flags: NodeCheckFlags,
    pub declaration_requires_scope_change: Tristate,
    pub has_reported_statement_in_ambient_context: bool,
}

#[derive(Debug, Default)]
pub struct SymbolNodeLinks {
    pub resolved_symbol: Option<Arc<Symbol>>,
}

#[derive(Debug, Default)]
pub struct TypeNodeLinks {
    pub resolved_type: Option<Arc<Type>>,
    pub outer_type_parameters: Vec<Arc<Type>>,
}

#[derive(Debug, Default)]
pub struct EnumMemberLinks {
    pub value: evaluator::EvalResult,
}

#[derive(Debug, Default)]
pub struct AssertionLinks {
    pub expr_type: Option<Arc<Type>>,
}

#[derive(Debug, Default)]
pub struct SourceFileLinks {
    pub type_checked: bool,
    pub unused_checked: bool,
    pub external_helpers_module: Option<Arc<Symbol>>,
    pub requested_external_emit_helpers: ExternalEmitHelpers,
    pub local_jsx_namespace: String,
    pub local_jsx_fragment_namespace: String,
    pub local_jsx_factory: Option<Arc<Node>>,
    pub local_jsx_fragment_factory: Option<Arc<Node>>,
    pub jsx_fragment_type: Option<Arc<Type>>,
}

#[derive(Debug, Default)]
pub struct SignatureLinks {
    pub resolved_signature: Option<Arc<Signature>>,
    pub effects_signature: Option<Arc<Signature>>,
    pub decorator_signature: Option<Arc<Signature>>,
}

#[derive(Debug, Default)]
pub struct JsxElementLinks {
    pub attributes_type: Option<Arc<Type>>,
    pub tag_name: Option<Arc<Symbol>>,
}

/// Key for the accessible-symbol-chain cache. Mirrors Go's
/// `accessibleChainCacheKey` (types.go).
#[derive(Debug, Clone)]
pub struct AccessibleChainCacheKey {
    pub use_only_external_aliasing: bool,
    pub location: Option<Arc<Node>>,
    pub meaning: SymbolFlags,
}

impl PartialEq for AccessibleChainCacheKey {
    fn eq(&self, other: &Self) -> bool {
        self.use_only_external_aliasing == other.use_only_external_aliasing
            && self.meaning == other.meaning
            && match (&self.location, &other.location) {
                (None, None) => true,
                (Some(a), Some(b)) => a.id() == b.id(),
                _ => false,
            }
    }
}

impl Eq for AccessibleChainCacheKey {}

impl std::hash::Hash for AccessibleChainCacheKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.use_only_external_aliasing.hash(state);
        self.meaning.bits().hash(state);
        match &self.location {
            Some(n) => n.id().hash(state),
            None => 0u64.hash(state),
        }
    }
}

/// Per-symbol links tracking accessible container relationships.
///
/// Mirrors Go's `ContainingSymbolLinks` (types.go).
#[derive(Debug, Default)]
pub struct ContainingSymbolLinks {
    /// Symbols of nodes which logically contain this one, cached by file
    /// the request is made within.
    pub extended_containers_by_file: HashMap<u64, Vec<Arc<Symbol>>>,
    /// Containers (other than the parent) which this symbol is aliased in.
    /// `None` means not yet computed; `Some(vec)` is the result (may be empty).
    pub extended_containers: Option<Vec<Arc<Symbol>>>,
    /// Cache for `getAccessibleSymbolChainEx`.
    pub accessible_chain_cache: HashMap<AccessibleChainCacheKey, Vec<Arc<Symbol>>>,
}

/// Per-declaration links used by the emit resolver to track visibility.
///
/// Mirrors Go's `DeclarationLinks` (emitresolver.go). `is_visible` is a
/// `Tristate` cached result of `isDeclarationVisible`.
#[derive(Debug, Default)]
pub struct DeclarationLinks {
    pub is_visible: Tristate,
}

/// Per-source-file links used by the emit resolver.
///
/// Mirrors Go's `DeclarationFileLinks` (emitresolver.go). `aliases_marked`
/// records whether `PrecalculateDeclarationEmitVisibility` has already run
/// the alias marking visitor over this file.
#[derive(Debug, Default)]
pub struct DeclarationFileLinks {
    pub aliases_marked: bool,
}

/// Result of a symbol accessibility / entity-name visibility query.
///
/// Mirrors Go's `printer.SymbolAccessibilityResult`. `PartialEq` is
/// intentionally not derived because `Node` is not `Eq` (it carries
/// interior-mutable state).
#[derive(Debug, Clone, Default)]
pub struct SymbolAccessibilityResult {
    pub accessibility: SymbolAccessibility,
    /// Aliases that must be marked visible for the reference to serialize.
    pub aliases_to_make_visible: Vec<Arc<crate::ast::Node>>,
    pub error_symbol_name: String,
    pub error_module_name: String,
    pub error_node: Option<Arc<crate::ast::Node>>,
}

/// The accessibility of a symbol relative to an enclosing declaration.
///
/// Mirrors Go's `printer.SymbolAccessibility`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SymbolAccessibility {
    #[default]
    Accessible,
    NotAccessible,
    CannotBeNamed,
    NotResolved,
}

// ────────────────────────────────────────────────────────────────────────────
// CheckMode
// ────────────────────────────────────────────────────────────────────────────

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct CheckMode: u32 {
        const Normal               = 0;
        const Contextual           = 1 << 0;
        const Inferential          = 1 << 1;
        const SkipContextSensitive = 1 << 2;
        const SkipGenericFunctions = 1 << 3;
        const IsForSignatureHelp   = 1 << 4;
        const RestBindingElement   = 1 << 5;
        const TypeOnly             = 1 << 6;
        const ForceTuple           = 1 << 7;
    }
}

// ────────────────────────────────────────────────────────────────────────────
// WideningKind
// ────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WideningKind {
    #[default]
    Normal,
    FunctionReturn,
    GeneratorNext,
    GeneratorYield,
}

// ────────────────────────────────────────────────────────────────────────────
// TypeSystemPropertyName
// ────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(i32)]
pub enum TypeSystemPropertyName {
    #[default]
    Type,
    ResolvedBaseConstructorType,
    DeclaredType,
    ResolvedReturnType,
    ResolvedBaseConstraint,
    ResolvedTypeArguments,
    ResolvedBaseTypes,
    WriteType,
    InitializerIsUndefined,
    AliasTarget,
}

// ────────────────────────────────────────────────────────────────────────────
// CachedTypeKind
// ────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(i32)]
pub enum CachedTypeKind {
    #[default]
    LiteralUnionBaseType,
    IndexType,
    StringIndexType,
    EquivalentBaseType,
    ApparentType,
    AwaitedType,
    EvolvingArrayType,
    ArrayLiteralType,
    PermissiveInstantiation,
    RestrictiveInstantiation,
    RestrictiveTypeParameter,
    IndexedAccessForReading,
    IndexedAccessForWriting,
    Widened,
    RegularObjectLiteral,
    PromisedTypeOfPromise,
    DefaultOnlyType,
    SyntheticType,
    DecoratorContext,
    DecoratorContextStatic,
    DecoratorContextPrivate,
    DecoratorContextPrivateStatic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CachedTypeKey {
    pub kind: CachedTypeKind,
    pub type_id: TypeId,
}

// ────────────────────────────────────────────────────────────────────────────
// RelationComparisonResult
// ────────────────────────────────────────────────────────────────────────────

pub type RelationComparisonResult = u32;

/// Cache key for `isEnumTypeRelatedTo`, indexing a `(source, target)` enum
/// symbol pair. Mirrors Go's `EnumRelationKey` (relater.go).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EnumRelationKey {
    pub source_id: u64,
    pub target_id: u64,
}

// ────────────────────────────────────────────────────────────────────────────
// CacheHashKey
// ────────────────────────────────────────────────────────────────────────────

/// A 128-bit hash key used for type caching.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct CacheHashKey {
    pub hi: u64,
    pub lo: u64,
}

impl CacheHashKey {
    pub fn new(hi: u64, lo: u64) -> Self {
        Self { hi, lo }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ternary_and_or() {
        assert_eq!(Ternary::True.and(Ternary::False), Ternary::False);
        assert_eq!(Ternary::True.or(Ternary::False), Ternary::True);
        assert_eq!(Ternary::Unknown.and(Ternary::Maybe), Ternary::Unknown);
        assert_eq!(Ternary::Unknown.or(Ternary::Maybe), Ternary::Maybe);
        assert_eq!(!Ternary::True, Ternary::False);
        assert_eq!(!Ternary::False, Ternary::True);
        assert_eq!(!Ternary::Unknown, Ternary::Unknown);
    }

    #[test]
    fn type_flags_composites() {
        assert!(TYPE_FLAGS_LITERAL.contains(TypeFlags::StringLiteral));
        assert!(TYPE_FLAGS_LITERAL.contains(TypeFlags::NumberLiteral));
        assert!(TYPE_FLAGS_NULLABLE.contains(TypeFlags::Undefined));
        assert!(TYPE_FLAGS_NULLABLE.contains(TypeFlags::Null));
        assert!(TYPE_FLAGS_STRING_LIKE.contains(TypeFlags::String));
        assert!(TYPE_FLAGS_STRING_LIKE.contains(TypeFlags::StringLiteral));
        assert!(TYPE_FLAGS_UNION_OR_INTERSECTION.contains(TypeFlags::Union));
        assert!(TYPE_FLAGS_UNION_OR_INTERSECTION.contains(TypeFlags::Intersection));
    }

    #[test]
    fn object_flags_composites() {
        assert!(OBJECT_FLAGS_CLASS_OR_INTERFACE.contains(ObjectFlags::Class));
        assert!(OBJECT_FLAGS_CLASS_OR_INTERFACE.contains(ObjectFlags::Interface));
    }

    #[test]
    fn signature_flags_propagating() {
        assert!(SIGNATURE_FLAGS_PROPAGATING_FLAGS.contains(SignatureFlags::HasRestParameter));
        assert!(SIGNATURE_FLAGS_PROPAGATING_FLAGS.contains(SignatureFlags::Construct));
        assert!(!SIGNATURE_FLAGS_PROPAGATING_FLAGS.contains(SignatureFlags::IsInnerCallChain));
    }

    #[test]
    fn literal_value_to_string() {
        assert_eq!(
            LiteralValue::String("hello".to_string()).to_string(),
            "\"hello\""
        );
        assert_eq!(LiteralValue::Boolean(true).to_string(), "true");
        assert_eq!(LiteralValue::Boolean(false).to_string(), "false");
        assert_eq!(LiteralValue::None.to_string(), "");
    }

    #[test]
    fn type_data_pattern_matching() {
        let t = Type::new(
            TypeFlags::String,
            TypeData::Intrinsic(IntrinsicTypeData {
                intrinsic_name: "string".to_string(),
            }),
        );
        assert!(t.is_string());
        assert!(!t.is_union());
        assert_eq!(t.intrinsic_name(), Some("string"));
    }

    #[test]
    fn structured_type_call_construct_signatures() {
        let mut structured = StructuredTypeData::default();
        structured.call_signature_count = 2;
        // Add 3 signatures: 2 call + 1 construct
        // (In real code these would be real signatures)
        structured.signatures = vec![
            Arc::new(Signature::new()),
            Arc::new(Signature::new()),
            Arc::new(Signature::new()),
        ];
        assert_eq!(structured.call_signatures().len(), 2);
        assert_eq!(structured.construct_signatures().len(), 1);
    }

    #[test]
    fn cache_hash_key() {
        let k1 = CacheHashKey::new(1, 2);
        let k2 = CacheHashKey::new(1, 2);
        let k3 = CacheHashKey::new(3, 4);
        assert_eq!(k1, k2);
        assert_ne!(k1, k3);
    }
}
