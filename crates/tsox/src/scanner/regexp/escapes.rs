use super::RegExpParser;
use super::{
    DecimalEscapeValue, GroupNameReference, decode_rune_at, is_ascii_letter, is_digit,
    is_hex_digit, is_octal_digit,
};
use crate::diagnostics;
use crate::scanner::is_identifier_part;
mod reg_exp_parser;
mod reg_exp_parser_2;
#[allow(unused_imports)]
pub use reg_exp_parser::*;
#[allow(unused_imports)]
pub use reg_exp_parser_2::*;
