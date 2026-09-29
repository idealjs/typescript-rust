#![allow(unused_imports, dead_code, unused_variables)]

use crate::execute::version::CommandLineResult;
use crate::execute::version::ExitStatus;

impl Default for CommandLineResult {
    fn default() -> Self {
        CommandLineResult {
            status: ExitStatus::Success,
            watcher: None,
        }
    }
}
