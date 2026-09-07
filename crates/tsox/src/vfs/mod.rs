mod fs;
mod fs_impl;
mod in_memory;
mod os_fs;
mod types;

pub use fs::FS;
pub use in_memory::InMemoryFS;
pub use os_fs::OsFS;
pub use types::*;

pub mod cachedvfs;
pub mod vfsmatch;

#[cfg(test)]
mod cachedvfs_tests;
#[cfg(test)]
mod iovfs_tests;
#[cfg(test)]
mod osvfs_tests;
#[cfg(test)]
mod vfsmatch_tests;
#[cfg(test)]
mod vfsmock_tests;
#[cfg(test)]
mod vfstest_tests;

#[cfg(test)]
mod tests;
