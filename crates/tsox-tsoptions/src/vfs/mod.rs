pub(crate) mod fs;
pub(crate) mod fs_impl;
pub(crate) mod in_memory;
pub(crate) mod os_fs;
pub(crate) mod types;

pub use fs::FS;
pub use in_memory::InMemoryFS;
pub use os_fs::OsFS;
pub use types::*;

pub mod cachedvfs;
pub mod vfsmatch;

#[cfg(test)]
pub(crate) mod cachedvfs_tests;
#[cfg(test)]
pub(crate) mod iovfs_tests;
#[cfg(test)]
pub(crate) mod osvfs_tests;
#[cfg(test)]
pub(crate) mod vfsmatch_tests;
#[cfg(test)]
pub(crate) mod vfsmock_tests;
#[cfg(test)]
pub(crate) mod vfstest_tests;

#[cfg(test)]
pub(crate) mod tests;
pub(crate) mod vfsmatch_impl_chunk;
pub(crate) mod vfsmatch_is_package_folder;
pub(crate) mod vfsmatch_unlimited_depth;
