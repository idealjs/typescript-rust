use crate::ast::{Node, Symbol, SymbolFlags, SymbolTable};
use crate::core::tristate::Tristate;
use crate::evaluator;
use crate::jsnum;
use bitflags::bitflags;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, OnceLock};
mod alias_symbol_links;
mod cached_type_kind;
mod impl_chunk;
mod impl_chunk_2;
mod impl_chunk_3;
mod type_flags_instantiable_non_primitive;
mod type_id;
#[allow(unused_imports)]
pub use alias_symbol_links::*;
#[allow(unused_imports)]
pub use cached_type_kind::*;
#[allow(unused_imports)]
pub use impl_chunk::*;
#[allow(unused_imports)]
pub use impl_chunk_2::*;
#[allow(unused_imports)]
pub use impl_chunk_3::*;
#[allow(unused_imports)]
pub use type_flags_instantiable_non_primitive::*;
#[allow(unused_imports)]
pub use type_id::*;
#[cfg(test)]
mod tests;
