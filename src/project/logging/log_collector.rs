use std::fmt;

use super::logger::Logger;

pub trait LogCollector: Logger + fmt::Display {}
