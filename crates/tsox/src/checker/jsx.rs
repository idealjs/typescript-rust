#![allow(dead_code)]
bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
    pub struct JsxFlags: u32 {
        const INTRINSIC_NAMED_ELEMENT = 1 << 0;
        const INTRINSIC_INDEXED_ELEMENT = 1 << 1;
    }
}
mod impl_chunk;
mod impl_chunk_2;
mod parse_isolated_entity_name;
#[allow(unused_imports)]
pub use impl_chunk::*;
#[allow(unused_imports)]
pub use impl_chunk_2::*;
#[allow(unused_imports)]
pub use parse_isolated_entity_name::*;
