pub(crate) use super::{
    NodeResolutionFeatures, PackageId, ResolvedModule, ResolvedTypeReferenceDirective,
    mangle_scoped_package_name, parse_package_name,
};
pub(crate) use crate::packagejson;
pub(crate) use crate::vfs::FS;
pub(crate) use bitflags::bitflags;
pub(crate) use std::collections::HashMap;
pub(crate) use std::sync::{Arc, Mutex};
pub(crate) use tsox_core::core::compiler_options::CompilerOptions;
pub(crate) use tsox_core::core::compiler_options::ModuleKind;
pub(crate) use tsox_core::core::compiler_options::ModuleResolutionKind;
pub(crate) use tsox_core::core::compiler_options::ResolutionMode;
pub(crate) use tsox_core::tspath;
bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct Extensions: i32 {
        const TYPESCRIPT     = 1;
        const JAVASCRIPT     = 1 << 1;
        const DECLARATION    = 1 << 2;
        const JSON           = 1 << 3;
    }
}
#[allow(unused_imports)]
pub use crate::module::resolver_impl_chunk_3::*;
#[allow(unused_imports)]
pub use crate::module::resolver_impl_chunk_4::*;
#[allow(unused_imports)]
pub use crate::module::resolver_resolution_state_5::*;
#[allow(unused_imports)]
pub use crate::module::resolver_resolution_state_6::*;
#[allow(unused_imports)]
pub use crate::module::resolver_resolution_state_7::*;
#[allow(unused_imports)]
pub use crate::module::resolver_resolution_state_8::*;
#[allow(unused_imports)]
pub use crate::module::resolver_resolution_state_9::*;
