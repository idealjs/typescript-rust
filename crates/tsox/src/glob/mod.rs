mod glob;
#[allow(unused_imports)]
pub use glob::*;
mod element;
mod matcher;
mod parse;

use element::Element;
#[cfg(test)]
mod tests;
