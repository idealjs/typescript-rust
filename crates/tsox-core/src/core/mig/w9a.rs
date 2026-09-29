#![allow(unused_imports)]

//! w9a: core 余量收尾批(w9)

pub(crate) fn next_arena_size(size: usize) -> usize {
    let size = size.max(1);
    size.saturating_mul(2).min(256)
}
