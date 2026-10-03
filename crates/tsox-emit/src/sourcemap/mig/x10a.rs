#![allow(dead_code, unused_imports, unused_variables)]

use super::super::decoder::MappingsDecoder;
use super::super::mapping::Mapping;

impl<'a> MappingsDecoder<'a> {
    pub fn set_error_and_stop_iterating(&mut self, error: &str) -> (Option<Mapping>, bool) { ::tsox_core::fntrace::enter("set_error_and_stop_iterating"); 
        self.set_error(error);
        self.stop_iterating()
    }
}
