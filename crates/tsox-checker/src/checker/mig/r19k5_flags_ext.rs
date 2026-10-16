use crate::checker::types_type_id::TypeFlags;

pub trait TypeFlagsExt {
    const StructuredType: TypeFlags;
    const UnionOrIntersection: TypeFlags;
    const StringOrNumberLiteralOrUnique: TypeFlags;
}

impl TypeFlagsExt for TypeFlags {
    const StructuredType: TypeFlags = TypeFlags::Object
        .union(TypeFlags::Union)
        .union(TypeFlags::Intersection)
        .union(TypeFlags::Index)
        .union(TypeFlags::IndexedAccess)
        .union(TypeFlags::Conditional)
        .union(TypeFlags::Substitution)
        .union(TypeFlags::TemplateLiteral)
        .union(TypeFlags::StringMapping);
    const UnionOrIntersection: TypeFlags =
        TypeFlags::Union.union(TypeFlags::Intersection);
    const StringOrNumberLiteralOrUnique: TypeFlags =
        TypeFlags::StringLiteral.union(TypeFlags::NumberLiteral)
            .union(TypeFlags::UniqueESSymbol);
}
