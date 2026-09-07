#![allow(unused_imports)]

use super::*;
use std::sync::LazyLock;

use super::{FILE_NAME_MAP_A, FILE_NAME_MAP_B};

static FULL_MAP: LazyLock<&'static [(&'static str, &'static str)]> = LazyLock::new(|| {
    let v: Vec<(&'static str, &'static str)> = FILE_NAME_MAP_A
        .iter()
        .chain(FILE_NAME_MAP_B.iter())
        .copied()
        .collect();
    Box::leak(v.into_boxed_slice())
});

pub fn safe_file_name_to_type_name() -> &'static [(&'static str, &'static str)] {
    &FULL_MAP
}
