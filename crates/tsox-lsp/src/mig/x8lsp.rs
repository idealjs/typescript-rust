#![allow(dead_code, unused_imports, unused_variables)]

use tsox_core::tspath;
use tsox_tsoptions::vfs::FS;

pub fn nearest_existing_ancestor(fs: &dyn FS, dir: &str) -> Option<String> { ::tsox_core::fntrace::enter("nearest_existing_ancestor"); 
    let mut dir = dir.to_string();
    loop {
        if fs.directory_exists(&dir) {
            return Some(dir);
        }
        let parent = tspath::get_directory_path(&dir);
        if parent == dir {
            return None;
        }
        dir = parent;
    }
}
