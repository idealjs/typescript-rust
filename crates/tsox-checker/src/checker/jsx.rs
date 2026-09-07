#![allow(dead_code)]
bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
    pub struct JsxFlags: u32 {
        const INTRINSIC_NAMED_ELEMENT = 1 << 0;
        const INTRINSIC_INDEXED_ELEMENT = 1 << 1;
    }
}
#[allow(unused_imports)]
pub use crate::checker::jsx_impl_chunk::*;
#[allow(unused_imports)]
pub use crate::checker::jsx_impl_chunk_2::*;
#[allow(unused_imports)]
pub use crate::checker::jsx_parse_isolated_entity_name::*;
