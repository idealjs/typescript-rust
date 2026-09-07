#![allow(dead_code)]
use crate::ast::{Node, SyntaxKind};
use crate::checker::Checker;
use crate::diagnostics::messages_generated::*;
use std::sync::Arc;
bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
    pub struct JsxFlags: u32 {
        const INTRINSIC_NAMED_ELEMENT = 1 << 0;
        const INTRINSIC_INDEXED_ELEMENT = 1 << 1;
    }
}
use super::*;
mod checker;
mod checker_2;
mod checker_3;
#[allow(unused_imports)]
pub use checker::*;
#[allow(unused_imports)]
pub use checker_2::*;
#[allow(unused_imports)]
pub use checker_3::*;
