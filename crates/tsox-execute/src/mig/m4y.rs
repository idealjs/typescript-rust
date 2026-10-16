#![allow(unused_imports, dead_code, unused_variables)]

use crate::execute::version::CommandLineResult;
use crate::execute::version::ExitStatus;

impl Default for CommandLineResult {
    fn default() -> Self { ::tsox_core::fntrace::enter("default"); 
        CommandLineResult {
            status: ExitStatus::Success,
            watcher: None,
        }
    }
}
