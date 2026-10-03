#![allow(dead_code, unused_imports, unused_variables)]

pub fn matches_pattern_with_trailer(target: &str, name: &str) -> bool { ::tsox_core::fntrace::enter("matches_pattern_with_trailer"); 
    if target.ends_with('*') {
        return false;
    }
    let Some((before, after)) = target.split_once('*') else {
        return false;
    };
    name.starts_with(before) && name.ends_with(after)
}
