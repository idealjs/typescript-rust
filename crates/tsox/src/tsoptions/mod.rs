use crate::ast::diagnostic::Diagnostic;
use crate::core::compiler_options::{
    CompilerOptions, JsxEmit, ModuleDetectionKind, ModuleKind, ModuleResolutionKind, NewLineKind,
    ScriptTarget,
};
use crate::core::text::TextRange;
use crate::core::tristate::Tristate;
use crate::core::watch_options::{
    PollingKind, WatchDirectoryKind, WatchFileKind, WatchOptions, parse_polling_kind,
    parse_watch_directory_kind, parse_watch_file_kind,
};
use crate::diagnostics::{
    ARGUMENT_FOR_0_OPTION_MUST_BE_COLON_1, CANNOT_READ_FILE_0,
    CIRCULARITY_DETECTED_WHILE_RESOLVING_CONFIGURATION_COLON_0,
    COMPILER_OPTION_0_MAY_NOT_BE_USED_WITH_BUILD, COMPILER_OPTION_0_MAY_ONLY_BE_USED_WITH_BUILD,
    NO_INPUTS_WERE_FOUND_IN_CONFIG_FILE_0_SPECIFIED_INCLUDE_PATHS_WERE_1_AND_EXCLUDE_PATHS_WERE_2,
    OPTION_0_CAN_ONLY_BE_SPECIFIED_IN_TSCONFIG_JSON_FILE_OR_SET_TO_FALSE_OR_NULL_ON_COMMAND_LINE,
    OPTION_0_CAN_ONLY_BE_SPECIFIED_IN_TSCONFIG_JSON_FILE_OR_SET_TO_NULL_ON_COMMAND_LINE,
    OPTION_0_REQUIRES_VALUE_TO_BE_GREATER_THAN_1, OPTIONS_0_AND_1_CANNOT_BE_COMBINED,
    UNKNOWN_BUILD_OPTION_0, UNKNOWN_BUILD_OPTION_0_DID_YOU_MEAN_1, UNKNOWN_COMPILER_OPTION_0,
    UNKNOWN_COMPILER_OPTION_0_DID_YOU_MEAN_1, UNTERMINATED_QUOTED_STRING_IN_RESPONSE_FILE_0,
    WATCH_OPTION_0_REQUIRES_A_VALUE_OF_TYPE_1, new_ad_hoc_message,
};
use crate::glob::Glob;
use crate::tspath;
use crate::vfs::FS;
use std::collections::{HashMap, HashSet};
mod apply_options;
mod build_options;
mod get_parsed_command_line_of_config_file_with_stack;
mod impl_chunk;
mod merge_compiler_options_with_skip;
mod option_kind;
mod options;
mod parse_command_line_worker;
mod parse_module_resolution;
mod parse_option_value;
mod resolve_relative_extends_path;
mod set_bool;
mod walk_and_match;
#[allow(unused_imports)]
pub use apply_options::*;
#[allow(unused_imports)]
pub use build_options::*;
#[allow(unused_imports)]
pub use get_parsed_command_line_of_config_file_with_stack::*;
#[allow(unused_imports)]
pub use impl_chunk::*;
#[allow(unused_imports)]
pub use merge_compiler_options_with_skip::*;
#[allow(unused_imports)]
pub use option_kind::*;
#[allow(unused_imports)]
pub use options::*;
#[allow(unused_imports)]
pub use parse_command_line_worker::*;
#[allow(unused_imports)]
pub use parse_module_resolution::*;
#[allow(unused_imports)]
pub use parse_option_value::*;
#[allow(unused_imports)]
pub use resolve_relative_extends_path::*;
#[allow(unused_imports)]
pub use set_bool::*;
#[allow(unused_imports)]
pub use walk_and_match::*;
#[cfg(test)]
mod tests;
