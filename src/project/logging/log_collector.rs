//! LogCollector trait (1:1 port of Go's `internal/project/logging/logcollector.go`).

use std::fmt;

use super::logger::Logger;

/// LogCollector is a Logger that also implements Display (String() in Go).
pub trait LogCollector: Logger + fmt::Display {}
