use std::sync::Arc;

use super::super::{BundledFS as WrappedFS, FS};

pub fn wrap_fs(fs: Arc<dyn FS>) -> Arc<dyn FS> { ::tsox_core::fntrace::enter("wrap_fs"); 
    wrap_fs_inner(fs)
}

fn wrap_fs_inner(fs: Arc<dyn FS>) -> Arc<dyn FS> { ::tsox_core::fntrace::enter("wrap_fs_inner"); 
    Arc::new(WrappedFS::new(fs))
}
