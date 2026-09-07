#![allow(unused_imports)]

use super::*;

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

pub type RelationComparisonResult = u32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EnumRelationKey {
    pub source_id: u64,
    pub target_id: u64,
}

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
