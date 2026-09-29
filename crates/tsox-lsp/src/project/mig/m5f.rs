#![allow(dead_code, unused_imports, unused_variables)]

use tsox_core::tspath;

pub fn perceived_os_root_length_for_watching(path_components: &[String]) -> usize {
    let length = path_components.len();
    if length <= 1 {
        return length;
    }
    if path_components[0].starts_with("//") {
        return 2;
    }
    let first = &path_components[0];
    let first_bytes = first.as_bytes();
    if first_bytes.len() == 3
        && tspath::is_volume_character(first_bytes[0])
        && first_bytes[1] == b':'
        && first_bytes[2] == b'/'
    {
        if path_components[1].eq_ignore_ascii_case("users") {
            return 3.min(length);
        }
        return 1;
    }
    if path_components[1] == "home" {
        return 3.min(length);
    }
    1
}
