#![allow(unused_imports)]

use super::*;
use std::sync::LazyLock;


pub static OPTIONS: LazyLock<Vec<OptionDecl>> = LazyLock::new(|| {
    let mut all = Vec::new();
    all.extend_from_slice(COMMAND_LINE_AND_STRICT);
    all.extend_from_slice(EMIT_AND_DIAGNOSTICS);
    all.extend_from_slice(RESOLUTION_AND_OUTPUT);
    all
});
