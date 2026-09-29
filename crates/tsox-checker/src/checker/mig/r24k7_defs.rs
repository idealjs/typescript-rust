use std::sync::Arc;

use crate::checker::checker::Checker;
use crate::checker::types::{IntersectionFlags, Type, TypeData, TypeFlags};

impl Checker {
    pub fn is_primitive_or_object_or_empty_type(&mut self, t: &Arc<Type>) -> bool {
        t.flags.intersects(TypeFlags::PRIMITIVE | TypeFlags::NON_PRIMITIVE) || self.is_empty_anonymous_object_type(t)
    }

    pub fn contains_missing_type(&self, t: &Arc<Type>) -> bool {
        Arc::ptr_eq(t, &self.missing_type)
            || t.flags.intersects(TypeFlags::UNION)
                && match &t.data {
                    TypeData::Union(u) => u
                        .union_or_intersection
                        .types
                        .first()
                        .is_some_and(|first| Arc::ptr_eq(first, &self.missing_type)),
                    _ => false,
                }
    }

    pub fn get_cross_product_intersections(&mut self, types: &[Arc<Type>], flags: IntersectionFlags) -> Vec<Arc<Type>> {
        let count = Checker::cross_product_union_size(types);
        let mut intersections: Vec<Arc<Type>> = Vec::new();
        for i in 0..count {
            let mut constituents: Vec<Arc<Type>> = types.to_vec();
            let mut n = i;
            for j in (0..types.len()).rev() {
                if types[j].flags.intersects(TypeFlags::UNION) {
                    if let TypeData::Union(u) = &types[j].data {
                        let source_types = &u.union_or_intersection.types;
                        let length = source_types.len();
                        constituents[j] = Arc::clone(&source_types[(n % length as u64) as usize]);
                        n /= length as u64;
                    }
                }
            }
            let t = self.get_intersection_type_ex(&constituents, flags, None);
            if !t.flags.intersects(TypeFlags::NEVER) {
                intersections.push(t);
            }
        }
        intersections
    }
}
