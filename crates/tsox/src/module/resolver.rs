use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use bitflags::bitflags;
use crate::core::compiler_options::{
    CompilerOptions, ModuleKind, ModuleResolutionKind, ResolutionMode,
};
use crate::packagejson;
use crate::tspath;
use crate::vfs::FS;
use super::{
    NodeResolutionFeatures, PackageId, ResolvedModule, ResolvedTypeReferenceDirective,
    mangle_scoped_package_name, parse_package_name,
};
bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct Extensions: i32 {
        const TYPESCRIPT     = 1;
        const JAVASCRIPT     = 1 << 1;
        const DECLARATION    = 1 << 2;
        const JSON           = 1 << 3;
    }
}
mod impl_chunk_3;
mod impl_chunk_4;
mod resolution_state_5;
mod resolution_state_6;
mod resolution_state_7;
mod resolution_state_8;
mod resolution_state_9;
#[allow(unused_imports)]
pub use impl_chunk_3::*;
#[allow(unused_imports)]
pub use impl_chunk_4::*;
#[allow(unused_imports)]
pub use resolution_state_5::*;
#[allow(unused_imports)]
pub use resolution_state_6::*;
#[allow(unused_imports)]
pub use resolution_state_7::*;
#[allow(unused_imports)]
pub use resolution_state_8::*;
#[allow(unused_imports)]
pub use resolution_state_9::*;
#[cfg(test)]
mod tests;
