#![allow(dead_code)]
pub(crate) use crate::checker::Checker;
pub(crate) use std::sync::Arc;
pub(crate) use tsox_core::diagnostics::messages_generated::*;
pub(crate) use tsox_frontend::ast::Node;
pub(crate) use tsox_frontend::ast::SyntaxKind;
bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
    pub struct JsxFlags: u32 {
        const INTRINSIC_NAMED_ELEMENT = 1 << 0;
        const INTRINSIC_INDEXED_ELEMENT = 1 << 1;
    }
}
pub(crate) use crate::checker::jsx::*;
#[allow(unused_imports)]
pub use crate::checker::jsx_impl_chunk_2_checker::*;
#[allow(unused_imports)]
pub use crate::checker::jsx_impl_chunk_2_checker_2::*;
#[allow(unused_imports)]
pub use crate::checker::jsx_impl_chunk_2_checker_3::*;
