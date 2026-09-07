#![allow(unused_imports)]

use super::*;

pub fn lookup_type_name(file_name: &str) -> Option<&'static str> {
    safe_file_name_to_type_name()
        .iter()
        .find(|(k, _)| *k == file_name)
        .map(|(_, v)| *v)
}
