#![allow(dead_code, unused_imports, unused_variables)]

use std::io::{self, ErrorKind};

pub type WalkFn<'a> = &'a dyn Fn(&str, bool) -> io::Result<()>;

pub const DT_UNKNOWN: u8 = 0;
pub const DT_DIR: u8 = 4;
pub const DT_LNK: u8 = 10;

pub struct UnixDirent {
    pub name: String,
    pub typ: u8,
}

pub fn walk_dir(dir: &str, recursive: bool, callback: WalkFn) -> io::Result<()> { crate::fntrace::enter("walk_dir"); 
    walk_dir_generic(dir, recursive, callback)
}

pub fn walk_dir_generic(dir: &str, recursive: bool, callback: WalkFn) -> io::Result<()> { crate::fntrace::enter("walk_dir_generic"); 
    let info = std::fs::symlink_metadata(dir)?;
    if !info.is_dir() {
        return Err(io::Error::from(ErrorKind::NotADirectory));
    }
    walk_dir_generic_visit(dir, recursive, callback)
}

pub fn walk_dir_generic_visit(dir: &str, recursive: bool, callback: WalkFn) -> io::Result<()> { crate::fntrace::enter("walk_dir_generic_visit"); 
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(err)
            if err.kind() == ErrorKind::PermissionDenied || err.kind() == ErrorKind::NotFound =>
        {
            return Ok(())
        }
        Err(err) => return Err(err),
    };
    callback(dir, true)?;
    let mut children: Vec<UnixDirent> = Vec::new();
    for entry in entries {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if name == "." || name == ".." {
            continue;
        }
        let typ = if entry.file_type()?.is_dir() {
            DT_DIR
        } else {
            DT_UNKNOWN
        };
        children.push(UnixDirent { name, typ });
    }
    children.sort_by(|a, b| a.name.cmp(&b.name));
    for ent in children {
        let path = format!("{}{}{}", dir, std::path::MAIN_SEPARATOR, ent.name);
        if ent.typ == DT_DIR {
            if recursive {
                walk_dir_generic_visit(&path, recursive, callback)?;
            } else {
                callback(&path, true)?;
            }
        } else {
            callback(&path, false)?;
        }
    }
    Ok(())
}

pub fn iterate_dir(dirname: &str, recursive: bool, callback: WalkFn) -> io::Result<()> { crate::fntrace::enter("iterate_dir"); 
    walk_dir_generic_visit(dirname, recursive, callback)
}

pub fn read_dir_entries(dir: &str) -> io::Result<Vec<UnixDirent>> { crate::fntrace::enter("read_dir_entries"); 
    let mut entries = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if name == "." || name == ".." {
            continue;
        }
        let file_type = entry.file_type()?;
        let typ = if file_type.is_dir() {
            DT_DIR
        } else if file_type.is_symlink() {
            DT_LNK
        } else {
            DT_UNKNOWN
        };
        entries.push(UnixDirent { name, typ });
    }
    Ok(entries)
}
