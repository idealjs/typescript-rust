use std::iter;

use crate::sourcemap::MappingsDecoder;

impl<'a> MappingsDecoder<'a> {
    pub fn values(&mut self) -> impl Iterator<Item = crate::sourcemap::Mapping> + '_ { ::tsox_core::fntrace::enter("values"); 
        iter::from_fn(move || self.next())
    }
}
