use crate::ast::diagnostic::Diagnostic;
use crate::bundled::{self, BundledFS};
use crate::compiler::{CompilerHost, CompilerHostImpl, Program, ProgramOptions};
use crate::core::compiler_options::CompilerOptions;
use crate::core::text::TextRange;
use crate::core::tristate::Tristate;
use crate::diagnostics::{
    A_TSCONFIG_JSON_FILE_IS_ALREADY_DEFINED_AT_COLON_0,
    CANNOT_FIND_A_TSCONFIG_JSON_FILE_AT_THE_CURRENT_DIRECTORY_COLON_0,
    CANNOT_FIND_A_TSCONFIG_JSON_FILE_AT_THE_SPECIFIED_DIRECTORY_COLON_0, CANNOT_READ_FILE_0,
    OPTION_BUILD_MUST_BE_THE_FIRST_COMMAND_LINE_ARGUMENT,
    OPTION_PROJECT_CANNOT_BE_MIXED_WITH_SOURCE_FILES_ON_A_COMMAND_LINE,
    OPTIONS_0_AND_1_CANNOT_BE_COMBINED,
    PROJECT_REFERENCES_MAY_NOT_FORM_A_CIRCULAR_GRAPH_CYCLE_DETECTED_COLON_0,
    THE_SPECIFIED_PATH_DOES_NOT_EXIST_COLON_0,
    X_TSCONFIG_JSON_IS_PRESENT_BUT_WILL_NOT_BE_LOADED_IF_FILES_ARE_SPECIFIED_ON_COMMANDLINE_USE_IGNORECONFIG_TO_SKIP_THIS_ERROR,
};
use crate::diagnosticwriter::{format_diagnostic, report_diagnostics};
use crate::incremental::{BuildInfo, compute_options_hash};
use crate::locale::Locale;
use crate::tsoptions::{
    BUILD_OPTIONS, BuildOptions, OPTIONS, OPTIONS_FOR_WATCH, OptionDecl, ParsedBuildCommandLine,
    ParsedCommandLine, get_parsed_command_line_of_config_file, parse_build_command_line,
    parse_command_line,
};
use crate::tspath;
use crate::vfs::{FS, OsFS};
use std::collections::HashSet;
use std::io::{IsTerminal, Write};
use std::sync::Arc;
use std::time::Instant;
mod build_project;
mod compute_options_signature;
mod generate_tsconfig;
mod perform_compilation;
mod show_config;
mod show_config_scalar_options;
mod show_config_bool_options;
mod tsc_compilation;
mod version;
#[allow(unused_imports)]
pub use build_project::*;
#[allow(unused_imports)]
pub use compute_options_signature::*;
#[allow(unused_imports)]
pub use generate_tsconfig::*;
#[allow(unused_imports)]
pub use perform_compilation::*;
#[allow(unused_imports)]
pub use show_config::*;
#[allow(unused_imports)]
pub(crate) use show_config_scalar_options::*;
#[allow(unused_imports)]
pub(crate) use show_config_bool_options::*;
#[allow(unused_imports)]
pub use tsc_compilation::*;
#[allow(unused_imports)]
pub use version::*;
mod watch;
#[cfg(test)]
mod tests;
