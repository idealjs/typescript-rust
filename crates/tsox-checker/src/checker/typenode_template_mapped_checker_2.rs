#![allow(unused_imports)]

use crate::checker::typenode_template_mapped::*;

impl Checker {
    pub(crate) fn make_union_two(&self, a: Arc<Type>, b: Arc<Type>) -> Arc<Type> { ::tsox_core::fntrace::enter("make_union_two"); 
        let propagated = self.get_propagating_flags_of_types(
            &[Arc::clone(&a), Arc::clone(&b)],
            crate::checker::types::TYPE_FLAGS_NULLABLE,
        );
        let mut t = Arc::new(Type::new(
            TypeFlags::Union,
            TypeData::Union(UnionTypeData {
                union_or_intersection: UnionOrIntersectionTypeData {
                    structured: StructuredTypeData::default(),
                    types: vec![a, b],
                },
                resolved_reduced_type: std::sync::OnceLock::new(),
                regular_type: std::sync::OnceLock::new(),
                origin: None,
                key_property_name: None,
                constituent_map: HashMap::new(),
            }),
        ));
        if let Some(t_mut) = Arc::get_mut(&mut t) {
            t_mut.object_flags |= propagated;
        }
        t
    }
}
