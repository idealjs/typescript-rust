#![allow(unused_imports)]

use crate::checker::types::*;

pub const OBJECT_FLAGS_REFERENCE: ObjectFlags = ObjectFlags::Reference;
pub const OBJECT_FLAGS_INSTANTIATED: ObjectFlags = ObjectFlags::Instantiated;
pub const OBJECT_FLAGS_MAPPED: ObjectFlags = ObjectFlags::Mapped;
pub const OBJECT_FLAGS_SINGLE_SIGNATURE_TYPE: ObjectFlags = ObjectFlags::SingleSignatureType;
pub const OBJECT_FLAGS_INSTANTIATION_EXPRESSION_TYPE: ObjectFlags =
    ObjectFlags::InstantiationExpressionType;

pub const TYPE_FLAGS_NOT_PRIMITIVE_UNION: TypeFlags = TypeFlags::from_bits_truncate(
    TypeFlags::Any.bits()
        | TypeFlags::Unknown.bits()
        | TypeFlags::Void.bits()
        | TypeFlags::Never.bits()
        | TypeFlags::Object.bits()
        | TypeFlags::Intersection.bits()
        | TypeFlags::Substitution.bits()
        | TypeFlags::TypeParameter.bits()
        | TypeFlags::IndexedAccess.bits()
        | TypeFlags::Conditional.bits()
        | TypeFlags::Index.bits()
        | TypeFlags::TemplateLiteral.bits()
        | TypeFlags::StringMapping.bits(),
);
