#![allow(unused_imports)]

use crate::checker::typenode_constructors::*;

const TYPE_FLAGS_NOT_PRIMITIVE_UNION: TypeFlags = TypeFlags::from_bits_truncate(
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

impl Checker {
    pub(crate) fn distribute_intersection_over_unions(
        &mut self,
        types: Vec<Arc<Type>>,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("distribute_intersection_over_unions"); 
        let (restarted, reduced) = self.intersect_unions_of_primitive_types(types);
        if reduced {
            return self.get_intersection_type(restarted);
        }
        let types = restarted;
        if types.iter().all(Self::is_union_with_undefined) {
            let stripped = types
                .iter()
                .map(|t| self.strip_undefined(t))
                .collect();
            let inner = self.get_intersection_type(stripped);
            return self.get_union_type(vec![inner, self.undefined_type()]);
        }
        if types.iter().all(Self::is_union_with_null) {
            let stripped = types.iter().map(|t| self.strip_null(t)).collect();
            let inner = self.get_intersection_type(stripped);
            return self.get_union_type(vec![inner, self.null_type()]);
        }
        let count = Self::cross_product_union_size(&types);
        if count >= 100_000 {
            return self.error_type();
        }
        let constituents = self.cross_product_intersections(&types, count as usize);
        let origin_needed = constituents
            .iter()
            .any(|t| t.flags.contains(TypeFlags::Intersection))
            && Self::constituent_count_of_types(&constituents)
                > Self::constituent_count_of_types(&types);
        let result = self.get_union_type(constituents);
        if origin_needed
            && matches!(result.data, TypeData::Union(_))
            && !matches!(&result.data, TypeData::Union(u) if u.origin.is_some())
        {
            let origin = new_denormalized_intersection(types);
            let ptr = Arc::as_ptr(&result) as *mut Type;
            unsafe {
                if let TypeData::Union(u) = &mut (*ptr).data {
                    u.origin = Some(origin);
                }
            }
        }
        result
    }

    fn cross_product_intersections(
        &mut self,
        types: &[Arc<Type>],
        count: usize,
    ) -> Vec<Arc<Type>> { ::tsox_core::fntrace::enter("cross_product_intersections"); 
        let mut intersections = Vec::with_capacity(count);
        for i in 0..count {
            let mut constituent = types.to_vec();
            let mut n = i;
            for j in (0..types.len()).rev() {
                if let TypeData::Union(u) = &types[j].data {
                    let source_types = &u.union_or_intersection.types;
                    let length = source_types.len();
                    constituent[j] = Arc::clone(&source_types[n % length]);
                    n /= length;
                }
            }
            let t = self.get_intersection_type(constituent);
            if !t.flags.contains(TypeFlags::Never) {
                intersections.push(t);
            }
        }
        intersections
    }

    fn is_union_with_undefined(t: &Arc<Type>) -> bool { ::tsox_core::fntrace::enter("is_union_with_undefined"); 
        matches!(&t.data, TypeData::Union(u)
            if u.union_or_intersection
                .types
                .first()
                .is_some_and(|f| f.flags.contains(TypeFlags::Undefined)))
    }

    fn is_union_with_null(t: &Arc<Type>) -> bool { ::tsox_core::fntrace::enter("is_union_with_null"); 
        matches!(&t.data, TypeData::Union(u)
            if u.union_or_intersection
                .types
                .iter()
                .take(2)
                .any(|f| f.flags.contains(TypeFlags::Null)))
    }

    fn strip_undefined(&mut self, t: &Arc<Type>) -> Arc<Type> { ::tsox_core::fntrace::enter("strip_undefined"); 
        let TypeData::Union(u) = &t.data else {
            return Arc::clone(t);
        };
        let kept = u
            .union_or_intersection
            .types
            .iter()
            .filter(|m| !m.flags.contains(TypeFlags::Undefined))
            .cloned()
            .collect();
        self.get_union_type(kept)
    }

    fn strip_null(&mut self, t: &Arc<Type>) -> Arc<Type> { ::tsox_core::fntrace::enter("strip_null"); 
        let TypeData::Union(u) = &t.data else {
            return Arc::clone(t);
        };
        let kept = u
            .union_or_intersection
            .types
            .iter()
            .filter(|m| !m.flags.contains(TypeFlags::Null))
            .cloned()
            .collect();
        self.get_union_type(kept)
    }

    fn constituent_count(t: &Arc<Type>) -> usize { ::tsox_core::fntrace::enter("constituent_count"); 
        if !t
            .flags
            .intersects(TYPE_FLAGS_UNION_OR_INTERSECTION)
            || t.alias.is_some()
        {
            return 1;
        }
        if let TypeData::Union(u) = &t.data
            && let Some(origin) = &u.origin
        {
            return Self::constituent_count(origin);
        }
        t.types()
            .map(|ms| ms.iter().map(Self::constituent_count).sum())
            .unwrap_or(1)
    }

    fn constituent_count_of_types(types: &[Arc<Type>]) -> usize { ::tsox_core::fntrace::enter("constituent_count_of_types"); 
        types.iter().map(Self::constituent_count).sum()
    }

    fn is_primitive_union(t: &Arc<Type>) -> bool { ::tsox_core::fntrace::enter("is_primitive_union"); 
        let TypeData::Union(u) = &t.data else {
            return false;
        };
        u.union_or_intersection
            .types
            .iter()
            .all(|m| !m.flags.intersects(TYPE_FLAGS_NOT_PRIMITIVE_UNION))
    }

    fn union_member_ptr(u: &Arc<Type>, t: &Arc<Type>) -> bool { ::tsox_core::fntrace::enter("union_member_ptr"); 
        let TypeData::Union(data) = &u.data else {
            return false;
        };
        data.union_or_intersection.types.iter().any(|m| m.id == t.id)
    }

    fn union_contains_type(&self, u: &Arc<Type>, t: &Arc<Type>) -> bool { ::tsox_core::fntrace::enter("union_contains_type"); 
        if Self::union_member_ptr(u, t) {
            return true;
        }
        let primitive = if t.flags.contains(TypeFlags::StringLiteral) {
            Some(self.string_type())
        } else if t.flags.intersects(TypeFlags::Enum | TypeFlags::NumberLiteral) {
            Some(self.number_type())
        } else if t.flags.contains(TypeFlags::BigIntLiteral) {
            Some(self.bigint_type())
        } else if t.flags.contains(TypeFlags::UniqueESSymbol) {
            Some(self.es_symbol_type())
        } else {
            None
        };
        primitive.is_some_and(|p| Self::union_member_ptr(u, &p))
    }

    fn each_union_contains(&self, union_types: &[Arc<Type>], t: &Arc<Type>) -> bool { ::tsox_core::fntrace::enter("each_union_contains"); 
        union_types.iter().all(|u| self.union_contains_type(u, t))
    }

    pub(crate) fn intersect_unions_of_primitive_types(
        &mut self,
        types: Vec<Arc<Type>>,
    ) -> (Vec<Arc<Type>>, bool) { ::tsox_core::fntrace::enter("intersect_unions_of_primitive_types"); 
        let Some(index) = types.iter().position(Self::is_primitive_union) else {
            return (types, false);
        };
        let mut union_types: Vec<Arc<Type>> = vec![Arc::clone(&types[index])];
        let mut rest: Vec<Arc<Type>> = Vec::with_capacity(types.len() - 1);
        for (i, t) in types.iter().enumerate() {
            if i == index {
                continue;
            }
            if Self::is_primitive_union(t) {
                union_types.push(Arc::clone(t));
            } else {
                rest.push(Arc::clone(t));
            }
        }
        if union_types.len() == 1 {
            return (types, false);
        }
        let mut checked: Vec<u32> = Vec::new();
        let mut result: Vec<Arc<Type>> = Vec::new();
        for u in &union_types {
            let TypeData::Union(data) = &u.data else {
                continue;
            };
            for t in &data.union_or_intersection.types {
                if checked.contains(&t.id) {
                    continue;
                }
                checked.push(t.id);
                if self.each_union_contains(&union_types, t) {
                    result.push(Arc::clone(t));
                }
            }
        }
        let merged = self.get_union_type(result);
        let mut out: Vec<Arc<Type>> = Vec::with_capacity(rest.len() + 1);
        let (head, tail) = rest.split_at(index);
        out.extend(head.iter().cloned());
        out.push(merged);
        out.extend(tail.iter().cloned());
        (out, true)
    }
}

fn new_denormalized_intersection(types: Vec<Arc<Type>>) -> Arc<Type> { ::tsox_core::fntrace::enter("new_denormalized_intersection"); 
    Arc::new(Type::new(
        TypeFlags::Intersection,
        TypeData::Intersection(IntersectionTypeData {
            union_or_intersection: UnionOrIntersectionTypeData {
                structured: StructuredTypeData::default(),
                types,
            },
            resolved_apparent_type: std::sync::OnceLock::new(),
            unique_literal_filled_instantiation: std::sync::OnceLock::new(),
            resolved_properties: std::sync::OnceLock::new(),
        }),
    ))
}
