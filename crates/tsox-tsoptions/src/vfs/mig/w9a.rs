#![allow(unused_imports)]

//! w9a: osvfs 余量收尾批(w9)

pub(crate) fn os_fs_realpath(path: &str) -> String { ::tsox_core::fntrace::enter("os_fs_realpath"); 
    let _ = tsox_core::tspath::get_encoded_root_length(path);
    let orig = path;
    let path = from_slash(path);
    let path = match tsox_core::nativepath::mig::m6a::realpath(&path) {
        Ok(p) => p,
        Err(_) => return orig.to_string(),
    };
    let path = match abs(&path) {
        Ok(p) => p,
        Err(_) => return orig.to_string(),
    };
    tsox_core::tspath::normalize_slashes(&path)
}

fn from_slash(path: &str) -> String { ::tsox_core::fntrace::enter("from_slash"); 
    if std::path::MAIN_SEPARATOR == '/' {
        path.to_string()
    } else {
        path.replace('/', &std::path::MAIN_SEPARATOR.to_string())
    }
}

fn abs(path: &str) -> std::io::Result<String> { ::tsox_core::fntrace::enter("abs"); 
    let p = std::path::Path::new(path);
    if p.is_absolute() {
        return Ok(path.to_string());
    }
    let cwd = std::env::current_dir()?;
    Ok(cwd.join(p).to_string_lossy().to_string())
}
