pub(crate) use crate::vfs::FS;
pub(crate) use std::collections::{HashMap, HashSet};
pub(crate) use tsox_core::core::compiler_options::CompilerOptions;
pub(crate) use tsox_core::core::compiler_options::JsxEmit;
pub(crate) use tsox_core::core::compiler_options::ModuleDetectionKind;
pub(crate) use tsox_core::core::compiler_options::ModuleKind;
pub(crate) use tsox_core::core::compiler_options::ModuleResolutionKind;
pub(crate) use tsox_core::core::compiler_options::NewLineKind;
pub(crate) use tsox_core::core::compiler_options::ScriptTarget;
pub(crate) use tsox_core::core::text::TextRange;
pub(crate) use tsox_core::core::tristate::Tristate;
pub(crate) use tsox_core::core::watch_options::PollingKind;
pub(crate) use tsox_core::core::watch_options::WatchDirectoryKind;
pub(crate) use tsox_core::core::watch_options::WatchFileKind;
pub(crate) use tsox_core::core::watch_options::WatchOptions;
pub(crate) use tsox_core::core::watch_options::parse_polling_kind;
pub(crate) use tsox_core::core::watch_options::parse_watch_directory_kind;
pub(crate) use tsox_core::core::watch_options::parse_watch_file_kind;
pub(crate) use tsox_core::diagnostics::ARGUMENT_FOR_0_OPTION_MUST_BE_COLON_1;
pub(crate) use tsox_core::diagnostics::CANNOT_READ_FILE_0;
pub(crate) use tsox_core::diagnostics::CIRCULARITY_DETECTED_WHILE_RESOLVING_CONFIGURATION_COLON_0;
pub(crate) use tsox_core::diagnostics::COMPILER_OPTION_0_MAY_NOT_BE_USED_WITH_BUILD;
pub(crate) use tsox_core::diagnostics::COMPILER_OPTION_0_MAY_ONLY_BE_USED_WITH_BUILD;
pub(crate) use tsox_core::diagnostics::NO_INPUTS_WERE_FOUND_IN_CONFIG_FILE_0_SPECIFIED_INCLUDE_PATHS_WERE_1_AND_EXCLUDE_PATHS_WERE_2;
pub(crate) use tsox_core::diagnostics::OPTION_0_CAN_ONLY_BE_SPECIFIED_IN_TSCONFIG_JSON_FILE_OR_SET_TO_FALSE_OR_NULL_ON_COMMAND_LINE;
pub(crate) use tsox_core::diagnostics::OPTION_0_CAN_ONLY_BE_SPECIFIED_IN_TSCONFIG_JSON_FILE_OR_SET_TO_NULL_ON_COMMAND_LINE;
pub(crate) use tsox_core::diagnostics::OPTION_0_REQUIRES_VALUE_TO_BE_GREATER_THAN_1;
pub(crate) use tsox_core::diagnostics::OPTIONS_0_AND_1_CANNOT_BE_COMBINED;
pub(crate) use tsox_core::diagnostics::UNKNOWN_BUILD_OPTION_0;
pub(crate) use tsox_core::diagnostics::UNKNOWN_BUILD_OPTION_0_DID_YOU_MEAN_1;
pub(crate) use tsox_core::diagnostics::UNKNOWN_COMPILER_OPTION_0;
pub(crate) use tsox_core::diagnostics::UNKNOWN_COMPILER_OPTION_0_DID_YOU_MEAN_1;
pub(crate) use tsox_core::diagnostics::UNTERMINATED_QUOTED_STRING_IN_RESPONSE_FILE_0;
pub(crate) use tsox_core::diagnostics::WATCH_OPTION_0_REQUIRES_A_VALUE_OF_TYPE_1;
pub(crate) use tsox_core::diagnostics::new_ad_hoc_message;
pub(crate) use tsox_core::glob::Glob;
pub(crate) use tsox_frontend::ast::diagnostic::Diagnostic;
pub(crate) mod apply_options;
pub(crate) mod build_options;
pub(crate) mod get_parsed_command_line_of_config_file_with_stack;
pub(crate) mod impl_chunk;
pub(crate) mod merge_compiler_options_with_skip;
pub(crate) mod option_kind;
pub(crate) mod options;
pub(crate) mod parse_command_line_worker;
pub(crate) mod parse_module_resolution;
pub(crate) mod parse_option_value;
pub(crate) mod resolve_relative_extends_path;
pub(crate) mod set_bool;
pub(crate) mod walk_and_match;
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
pub(crate) mod options_command_line_and_strict;
pub(crate) mod options_emit_and_diagnostics;
pub(crate) mod options_options;
pub(crate) mod options_resolution_and_output;
#[cfg(test)]
pub(crate) mod tests;

pub fn implied_node_format_of_file(
    file_name: &str,
    read_file: &dyn Fn(&str) -> Option<String>,
) -> tsox_core::core::compiler_options::ModuleKind {
    use tsox_core::core::compiler_options::ModuleKind;
    let lower = file_name.to_ascii_lowercase();
    if lower.ends_with(".mts") || lower.ends_with(".mjs") || lower.ends_with(".mjsx") {
        return ModuleKind::ESNext;
    }
    if lower.ends_with(".cts") || lower.ends_with(".cjs") || lower.ends_with(".cjsx") {
        return ModuleKind::CommonJS;
    }

    let mut dir = tsox_core::tspath::get_directory_path(file_name);
    loop {
        let pkg = tsox_core::tspath::combine_paths(&dir, &["package.json"]);
        if let Some(text) = read_file(&pkg)
            && let Ok(fields) = crate::packagejson::parse(&text)
            && let Some(ty) = fields.header_fields.r#type.get_value()
        {
            return if ty == "module" {
                ModuleKind::ESNext
            } else {
                ModuleKind::CommonJS
            };
        }
        let parent = tsox_core::tspath::get_directory_path(&dir);
        if parent == dir {
            return ModuleKind::CommonJS;
        }
        dir = parent;
    }
}
