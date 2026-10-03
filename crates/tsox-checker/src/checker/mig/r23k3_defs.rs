//! r23-k3 补充定义:KeyBuilder 哈希收口、IterationTypes 默认值、self 借用安全遍历辅助。
#![allow(unused_imports)]

use crate::checker::checker_checker_checker::Checker;
use crate::checker::mig::m1f_2::IterationTypes;
use crate::checker::types::KeyBuilder;
use crate::checker::types::{ObjectFlags, Type, TypeFlags};
use crate::checker::types_cached_type_kind::CacheHashKey;
use std::sync::Arc;
use tsox_frontend::ast::Node;

pub(crate) struct R23KeyBuilder {
    hi: u64,
    lo: u64,
}

impl Default for R23KeyBuilder {
    fn default() -> Self { ::tsox_core::fntrace::enter("default"); 
        Self::new()
    }
}

impl R23KeyBuilder {
    pub fn new() -> R23KeyBuilder { ::tsox_core::fntrace::enter("new"); 
        R23KeyBuilder {
            hi: 0xcbf2_9ce4_8422_2325,
            lo: 0x9e37_79b9_7f4a_7c15,
        }
    }

    pub fn hash(&self) -> CacheHashKey { ::tsox_core::fntrace::enter("hash"); 
        CacheHashKey::new(self.hi, self.lo)
    }

    pub fn write_byte(&mut self, b: u8) { ::tsox_core::fntrace::enter("write_byte"); 
        self.hi = (self.hi ^ u64::from(b)).wrapping_mul(0x100_0000_01b3);
        self.lo = (self.lo ^ (u64::from(b) << 1 | 1)).wrapping_mul(0x100_0000_01b3);
    }

    pub fn write_u64(&mut self, v: u64) { ::tsox_core::fntrace::enter("write_u64"); 
        for b in v.to_le_bytes() {
            self.write_byte(b);
        }
    }

    pub fn write_u32(&mut self, v: u32) { ::tsox_core::fntrace::enter("write_u32"); 
        self.write_u64(u64::from(v));
    }

    pub fn write_int(&mut self, value: i32) { ::tsox_core::fntrace::enter("write_int"); 
        self.write_u64(value as i64 as u64);
    }

    pub fn write_type(&mut self, t: &Type) { ::tsox_core::fntrace::enter("write_type"); 
        self.write_u64(u64::from(t.id));
    }

    pub fn write_types(&mut self, types: &[Arc<Type>]) { ::tsox_core::fntrace::enter("write_types"); 
        self.write_u64(types.len() as u64);
        for t in types {
            self.write_type(t);
        }
    }

    pub fn write_alias(&mut self, alias: Option<&crate::checker::types::TypeAlias>) { ::tsox_core::fntrace::enter("write_alias"); 
        match alias {
            None => self.write_byte(0),
            Some(a) => {
                self.write_byte(1);
                match &a.symbol {
                    None => self.write_u64(0),
                    Some(s) => self.write_u64(Arc::as_ptr(s) as *const () as u64),
                }
                self.write_u64(a.type_arguments.len() as u64);
                for t in &a.type_arguments {
                    self.write_type(t);
                }
            }
        }
    }

    pub fn write_node(&mut self, node: Option<&Arc<Node>>) { ::tsox_core::fntrace::enter("write_node"); 
        if let Some(node) = node {
            self.write_u64(node.id());
        }
    }
}

impl Default for IterationTypes {
    fn default() -> IterationTypes { ::tsox_core::fntrace::enter("default"); 
        IterationTypes {
            yield_type: None,
            return_type: None,
            next_type: None,
        }
    }
}

pub(crate) fn some_type_self<F>(checker: &mut Checker, t: &Arc<Type>, f: F) -> bool
where
    F: Fn(&mut Checker, &Arc<Type>) -> bool,
{ ::tsox_core::fntrace::enter("some_type_self"); 
    if t.flags.intersects(TypeFlags::Union) {
        if let Some(types) = t.types() {
            return types.iter().any(|u| f(checker, u));
        }
    }
    f(checker, t)
}

pub(crate) fn map_type_self(
    checker: &mut Checker,
    t: &Arc<Type>,
    f: &mut dyn FnMut(&mut Checker, &Arc<Type>) -> Option<Arc<Type>>,
) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("map_type_self"); 
    if t.flags.contains(TypeFlags::Never) {
        return Some(Arc::clone(t));
    }
    if !t.flags.contains(TypeFlags::Union) {
        return f(checker, t);
    }
    let types: Vec<Arc<Type>> = t.types().unwrap_or(&[]).to_vec();
    let mut mapped_types: Vec<Arc<Type>> = Vec::new();
    let mut changed = false;
    for s in &types {
        let mapped = if s.flags.contains(TypeFlags::Union) {
            map_type_self(checker, s, f)
        } else {
            f(checker, s)
        };
        if !mapped.as_ref().is_some_and(|m| Arc::ptr_eq(m, s)) {
            changed = true;
        }
        if let Some(mapped) = mapped {
            mapped_types.push(mapped);
        }
    }
    if changed {
        if mapped_types.is_empty() {
            return None;
        }
        return Some(checker.get_union_type_ex(mapped_types, crate::checker::types::UnionReduction::Literal));
    }
    Some(Arc::clone(t))
}

pub(crate) fn filter_type_self<F>(checker: &mut Checker, t: &Arc<Type>, mut f: F) -> Arc<Type>
where
    F: FnMut(&mut Checker, &Arc<Type>) -> bool,
{ ::tsox_core::fntrace::enter("filter_type_self"); 
    if t.flags.intersects(TypeFlags::Union) {
        let types: Vec<Arc<Type>> = t.types().unwrap_or(&[]).to_vec();
        let filtered: Vec<Arc<Type>> = types
            .iter()
            .filter(|u| f(checker, u))
            .cloned()
            .collect();
        let same = types.len() == filtered.len()
            && types
                .iter()
                .zip(filtered.iter())
                .all(|(a, b)| Arc::ptr_eq(a, b));
        if same {
            return Arc::clone(t);
        }
        let origin = t.as_union_type().and_then(|u| u.origin.clone());
        let mut new_origin: Option<Arc<Type>> = None;
        if let Some(origin) = &origin {
            if origin.flags.intersects(TypeFlags::Union) {
                let origin_types: Vec<Arc<Type>> = origin.types().unwrap_or(&[]).to_vec();
                let origin_filtered: Vec<Arc<Type>> = origin_types
                    .iter()
                    .filter(|u| u.flags.intersects(TypeFlags::Union) || f(checker, u))
                    .cloned()
                    .collect();
                if origin_types.len() - origin_filtered.len() == types.len() - filtered.len() {
                    if origin_filtered.len() == 1 {
                        return Arc::clone(&origin_filtered[0]);
                    }
                    new_origin = Some(checker.new_union_type(ObjectFlags::None, &origin_filtered));
                }
            }
        }
        let object_flags = t.object_flags
            & (ObjectFlags::PrimitiveUnion | ObjectFlags::ContainsIntersections);
        return checker.get_union_type_from_sorted_list(
            filtered,
            object_flags,
            None,
            new_origin.as_ref(),
        );
    }
    if t.flags.intersects(TypeFlags::Never) || f(checker, t) {
        return Arc::clone(t);
    }
    checker.never_type()
}
