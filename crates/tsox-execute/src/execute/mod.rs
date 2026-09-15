pub(crate) use std::collections::HashSet;
pub(crate) use std::io::{IsTerminal, Write};
pub(crate) use std::sync::Arc;
pub(crate) use std::time::Instant;
pub(crate) use tsox_checker::bundled::BundledFS;
pub(crate) use tsox_compile::compiler::CompilerHost;
pub(crate) use tsox_compile::compiler::CompilerHostImpl;
pub(crate) use tsox_compile::compiler::Program;
pub(crate) use tsox_compile::compiler::ProgramOptions;
pub(crate) use tsox_compile::incremental::BuildInfo;
pub(crate) use tsox_compile::incremental::compute_options_hash;
pub(crate) use tsox_core::core::compiler_options::CompilerOptions;
pub(crate) use tsox_core::core::text::TextRange;
pub(crate) use tsox_core::core::tristate::Tristate;
pub(crate) use tsox_core::diagnostics::A_TSCONFIG_JSON_FILE_IS_ALREADY_DEFINED_AT_COLON_0;
pub(crate) use tsox_core::diagnostics::CANNOT_FIND_A_TSCONFIG_JSON_FILE_AT_THE_CURRENT_DIRECTORY_COLON_0;
pub(crate) use tsox_core::diagnostics::CANNOT_FIND_A_TSCONFIG_JSON_FILE_AT_THE_SPECIFIED_DIRECTORY_COLON_0;
pub(crate) use tsox_core::diagnostics::CANNOT_READ_FILE_0;
pub(crate) use tsox_core::diagnostics::OPTION_BUILD_MUST_BE_THE_FIRST_COMMAND_LINE_ARGUMENT;
pub(crate) use tsox_core::diagnostics::OPTION_PROJECT_CANNOT_BE_MIXED_WITH_SOURCE_FILES_ON_A_COMMAND_LINE;
pub(crate) use tsox_core::diagnostics::OPTIONS_0_AND_1_CANNOT_BE_COMBINED;
pub(crate) use tsox_core::diagnostics::PROJECT_REFERENCES_MAY_NOT_FORM_A_CIRCULAR_GRAPH_CYCLE_DETECTED_COLON_0;
pub(crate) use tsox_core::diagnostics::THE_SPECIFIED_PATH_DOES_NOT_EXIST_COLON_0;
pub(crate) use tsox_core::diagnostics::X_TSCONFIG_JSON_IS_PRESENT_BUT_WILL_NOT_BE_LOADED_IF_FILES_ARE_SPECIFIED_ON_COMMANDLINE_USE_IGNORECONFIG_TO_SKIP_THIS_ERROR;
pub(crate) use tsox_core::locale::Locale;
pub(crate) use tsox_frontend::ast::diagnostic::Diagnostic;
pub(crate) use tsox_frontend::diagnosticwriter::format_diagnostic;
pub(crate) use tsox_frontend::diagnosticwriter::report_diagnostics;
pub(crate) use tsox_tsoptions::tsoptions::BUILD_OPTIONS;
pub(crate) use tsox_tsoptions::tsoptions::BuildOptions;
pub(crate) use tsox_tsoptions::tsoptions::OPTIONS;
pub(crate) use tsox_tsoptions::tsoptions::OPTIONS_FOR_WATCH;
pub(crate) use tsox_tsoptions::tsoptions::OptionDecl;
pub(crate) use tsox_tsoptions::tsoptions::ParsedBuildCommandLine;
pub(crate) use tsox_tsoptions::tsoptions::ParsedCommandLine;
pub(crate) use tsox_tsoptions::tsoptions::get_parsed_command_line_of_config_file;
pub(crate) use tsox_tsoptions::tsoptions::parse_build_command_line;
pub(crate) use tsox_tsoptions::tsoptions::parse_command_line;
pub(crate) use tsox_tsoptions::vfs::FS;
pub(crate) use tsox_tsoptions::vfs::OsFS;
pub(crate) mod build_project;
pub(crate) mod compute_options_signature;
pub(crate) mod generate_tsconfig;
pub(crate) mod perform_compilation;
pub(crate) mod show_config;
pub(crate) mod show_config_bool_options;
pub(crate) mod show_config_scalar_options;
pub(crate) mod tsc_compilation;
pub(crate) mod version;
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
pub(crate) use show_config_bool_options::*;
#[allow(unused_imports)]
pub(crate) use show_config_scalar_options::*;
#[allow(unused_imports)]
pub use tsc_compilation::*;
#[allow(unused_imports)]
pub use version::*;
#[cfg(test)]
pub(crate) mod tests;
pub(crate) mod watch;
