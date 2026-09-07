#![allow(unused_imports)]

use super::*;

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
        const IsGenericTypeComputed                      = 1 << 22;
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

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct IndexFlags: u32 {
        const None              = 0;
        const StringsOnly       = 1 << 0;
        const NoIndexSignatures = 1 << 1;
        const NoReducibleCheck  = 1 << 2;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i8)]
pub enum Ternary {
    False = 0,
    Unknown = 1,
    Maybe = 3,
    True = -1,
}
