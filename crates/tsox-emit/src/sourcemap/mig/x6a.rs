#![allow(unused_imports)]
#![allow(dead_code)]

use crate::sourcemap::decoder::MappingsDecoder;

impl<'a> MappingsDecoder<'a> {
    pub(crate) fn has_reported_error(&self) -> bool {
        self.error().is_some()
    }
}
