#![allow(unused_imports)]
#![allow(dead_code)]

use super::super::parsing_context::Parser;

pub(crate) fn get_parser() -> Parser { ::tsox_core::fntrace::enter("get_parser"); 
    Parser::new_with_language_variant(String::new(), Default::default())
}
