#[allow(unused_imports)]
pub use crate::checker::types_alias_symbol_links::*;
#[allow(unused_imports)]
pub use crate::checker::types_cached_type_kind::*;
#[allow(unused_imports)]
pub use crate::checker::types_impl_chunk::*;
#[allow(unused_imports)]
pub use crate::checker::types_impl_chunk_2::*;
#[allow(unused_imports)]
pub use crate::checker::types_impl_chunk_3::*;
#[allow(unused_imports)]
pub use crate::checker::types_type_flags_instantiable_non_primitive::*;
#[allow(unused_imports)]
pub use crate::checker::types_type_id::*;
pub(crate) use bitflags::bitflags;
pub(crate) use std::collections::HashMap;
pub(crate) use std::sync::atomic::{AtomicU32, Ordering};
pub(crate) use std::sync::{Arc, OnceLock};
pub(crate) use tsox_core::core::tristate::Tristate;
pub(crate) use tsox_frontend::ast::Node;
pub(crate) use tsox_frontend::ast::Symbol;
pub(crate) use tsox_frontend::ast::SymbolFlags;
pub(crate) use tsox_frontend::ast::SymbolTable;
pub(crate) use crate::checker::checker_iteration::{IterationTypes, IterationUse};
pub use crate::checker::exports_union_reduction::UnionReduction;
pub(crate) use crate::checker::inference::InferenceContext;
pub(crate) use crate::checker::mig::m1f_2::IterationTypesResolver;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct IntersectionFlags(pub u32);

#[allow(non_upper_case_globals)]
impl IntersectionFlags {
    pub const None: IntersectionFlags = IntersectionFlags(0);
    pub const NO_SUPERTYPE_REDUCTION: IntersectionFlags = IntersectionFlags(1 << 0);
    pub const NO_CONSTRAINT_REDUCTION: IntersectionFlags = IntersectionFlags(1 << 1);

    pub const fn empty() -> IntersectionFlags {
        IntersectionFlags(0)
    }
    pub const fn bits(self) -> u32 {
        self.0
    }
    pub const fn intersects(self, other: IntersectionFlags) -> bool {
        (self.0 & other.0) != 0
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct TypeFacts: u32 {
        const TYPEOF_EQ_STRING = 1 << 0;
        const TYPEOF_EQ_NUMBER = 1 << 1;
        const TYPEOF_EQ_BIGINT = 1 << 2;
        const TYPEOF_EQ_BOOLEAN = 1 << 3;
        const TYPEOF_EQ_SYMBOL = 1 << 4;
        const TYPEOF_EQ_OBJECT = 1 << 5;
        const TYPEOF_EQ_FUNCTION = 1 << 6;
        const TYPEOF_EQ_HOST_OBJECT = 1 << 7;
        const TYPEOF_NE_STRING = 1 << 8;
        const TYPEOF_NE_NUMBER = 1 << 9;
        const TYPEOF_NE_BIGINT = 1 << 10;
        const TYPEOF_NE_BOOLEAN = 1 << 11;
        const TYPEOF_NE_SYMBOL = 1 << 12;
        const TYPEOF_NE_OBJECT = 1 << 13;
        const TYPEOF_NE_FUNCTION = 1 << 14;
        const TYPEOF_NE_HOST_OBJECT = 1 << 15;
        const EQ_UNDEFINED = 1 << 16;
        const EQ_NULL = 1 << 17;
        const EQ_UNDEFINED_OR_NULL = 1 << 18;
        const NE_UNDEFINED = 1 << 19;
        const NE_NULL = 1 << 20;
        const NE_UNDEFINED_OR_NULL = 1 << 21;
        const TRUTHY = 1 << 22;
        const FALSY = 1 << 23;
        const IS_UNDEFINED = 1 << 24;
        const IS_NULL = 1 << 25;
        const IS_UNDEFINED_OR_NULL = (1 << 24) | (1 << 25);
        const ALL = (1 << 27) - 1;
        const OR_FACTS_MASK = (1 << 6) | (1 << 13);
        const AND_FACTS_MASK = 0x07FF_DFBF;
    }
}

pub enum IterationTypeKind {
    YIELD,
    RETURN,
    NEXT,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemberOverrideStatus {
    None,
    NeedsOverride,
    HasInvalidOverride,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InstantiationExpressionKey {
    pub node_id: u64,
    pub type_id: TypeId,
}

pub struct KeyBuilder {
    hi: u64,
    lo: u64,
}

impl Default for KeyBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyBuilder {
    pub fn new() -> KeyBuilder {
        KeyBuilder {
            hi: 0xcbf2_9ce4_8422_2325,
            lo: 0x9e37_79b9_7f4a_7c15,
        }
    }

    pub fn write_byte(&mut self, b: u8) {
        self.hi = (self.hi ^ u64::from(b)).wrapping_mul(0x100_0000_01b3);
        self.lo = (self.lo ^ (u64::from(b) << 1 | 1)).wrapping_mul(0x100_0000_01b3);
    }

    pub fn write_u32(&mut self, v: u32) {
        self.write_u64(u64::from(v));
    }

    pub fn write_u64(&mut self, v: u64) {
        for b in v.to_le_bytes() {
            self.write_byte(b);
        }
    }

    pub fn write_type(&mut self, t: &Type) {
        self.write_u64(u64::from(t.id));
    }

    pub fn write_types(&mut self, types: &[Arc<Type>]) {
        self.write_u64(types.len() as u64);
        for t in types {
            self.write_type(t);
        }
    }

    pub fn write_alias(&mut self, alias: Option<&TypeAlias>) {
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

    pub fn hash(&self) -> crate::checker::types_cached_type_kind::CacheHashKey {
        crate::checker::types_cached_type_kind::CacheHashKey::new(self.hi, self.lo)
    }
}

pub struct StructuredType {
    pub typ: Arc<Type>,
    pub members: SymbolTable,
    pub properties: Vec<Arc<Symbol>>,
    pub signatures: Vec<Arc<Signature>>,
    pub call_signature_count: usize,
    pub index_infos: Vec<Arc<IndexInfo>>,
    pub object_type_without_abstract_construct_signatures: OnceLock<Arc<Type>>,
}

impl StructuredType {
    pub fn as_type(&self) -> Arc<Type> {
        Arc::clone(&self.typ)
    }

    pub fn call_signatures(&self) -> &[Arc<Signature>] {
        &self.signatures[..self.call_signature_count]
    }

    pub fn construct_signatures(&self) -> &[Arc<Signature>] {
        &self.signatures[self.call_signature_count..]
    }
}
