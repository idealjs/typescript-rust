use std::fmt;

use crate::project::logging_logger::Logger;

pub trait LogCollector: Logger + fmt::Display {}
