use std::collections::HashSet;
use std::sync::LazyLock;

use super::{CODE_AND_NAME_HALF_A, CODE_AND_NAME_HALF_B};


pub static SCRIPT_VALUES: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    HashSet::from_iter(
        CODE_AND_NAME_HALF_A
            .iter()
            .chain(CODE_AND_NAME_HALF_B.iter())
            .copied(),
    )
});
