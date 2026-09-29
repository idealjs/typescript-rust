#![allow(dead_code, unused_imports, unused_variables)]

use super::*;

use tsox_core::diagnostics::{self, Message};
use tsox_frontend::ast::diagnostic::Diagnostic;
use tsox_frontend::ast::mig::m3d_2::new_compiler_diagnostic;
use tsox_frontend::ast::node_node::Node;
use tsox_frontend::ast::node_source_file::SourceFile;

use super::m5h::{AlternateModeDiagnostics, CommandLineParser};
use super::m5h_2::{get_name_map_from_list, NameMap};

pub fn create_diagnostic_for_node_in_source_file(
    source_file: &SourceFile,
    node: &Node,
    message: Message,
    args: Vec<String>,
) -> Diagnostic {
    let _ = (source_file, node);
    new_compiler_diagnostic(message, args)
}

pub fn create_diagnostic_for_node_in_source_file_or_compiler_diagnostic(
    source_file: Option<&SourceFile>,
    node: Option<&Node>,
    message: Message,
    args: Vec<String>,
) -> Diagnostic {
    if let (Some(source_file), Some(node)) = (source_file, node) {
        return create_diagnostic_for_node_in_source_file(source_file, node, message, args);
    }
    new_compiler_diagnostic(message, args)
}

impl CommandLineParser {
    pub fn create_unknown_option_error(
        &self,
        unknown_option: &str,
        unknown_option_error_text: &str,
        node: Option<&Node>,
        source_file: Option<&SourceFile>,
    ) -> Diagnostic {
        let options_name_map = get_name_map_from_list(self.options_declarations());
        create_unknown_option_error(
            unknown_option,
            Some(self.unknown_option_diagnostic()),
            Some(unknown_option_error_text),
            node,
            source_file,
            self.alternate_mode(),
            Some(self.unknown_did_you_mean_diagnostic()),
            Some(&options_name_map),
        )
    }
}

pub fn create_unknown_option_error(
    unknown_option: &str,
    unknown_option_diagnostic: Option<Message>,
    unknown_option_error_text: Option<&str>,
    node: Option<&Node>,
    source_file: Option<&SourceFile>,
    alternate_mode: Option<&AlternateModeDiagnostics>,
    unknown_did_you_mean_diagnostic: Option<Message>,
    options_name_map: Option<&NameMap>,
) -> Diagnostic {
    if let Some(alternate_mode) = alternate_mode {
        if let Some(other_option) = alternate_mode
            .options_name_map
            .get(&unknown_option.to_lowercase())
        {
            let diagnostic = if other_option.name == "build" {
                diagnostics::OPTION_BUILD_MUST_BE_THE_FIRST_COMMAND_LINE_ARGUMENT
            } else {
                alternate_mode.diagnostic
            };
            return create_diagnostic_for_node_in_source_file_or_compiler_diagnostic(
                source_file,
                node,
                diagnostic,
                vec![unknown_option.to_string()],
            );
        }
    }
    let unknown_option_error_text = match unknown_option_error_text {
        Some(text) if !text.is_empty() => text.to_string(),
        _ => unknown_option.to_string(),
    };
    if let (Some(unknown_did_you_mean_diagnostic), Some(options_name_map)) =
        (unknown_did_you_mean_diagnostic, options_name_map)
    {
        if let Some(possible_option) =
            options_name_map.get_spelling_suggestion(unknown_option)
        {
            return create_diagnostic_for_node_in_source_file_or_compiler_diagnostic(
                source_file,
                node,
                unknown_did_you_mean_diagnostic,
                vec![unknown_option_error_text, possible_option.name.to_string()],
            );
        }
    }
    create_diagnostic_for_node_in_source_file_or_compiler_diagnostic(
        source_file,
        node,
        unknown_option_diagnostic.unwrap(),
        vec![unknown_option_error_text],
    )
}

pub fn extra_key_diagnostics(s: &str) -> Option<Message> {
    match s {
        "compilerOptions" => Some(diagnostics::UNKNOWN_COMPILER_OPTION_0),
        "watchOptions" => Some(diagnostics::UNKNOWN_WATCH_OPTION_0),
        "typeAcquisition" => Some(diagnostics::UNKNOWN_TYPE_ACQUISITION_OPTION_0),
        "buildOptions" => Some(diagnostics::UNKNOWN_BUILD_OPTION_0),
        _ => None,
    }
}

pub fn extra_key_did_you_mean_diagnostics(s: &str) -> Option<Message> {
    match s {
        "compilerOptions" => Some(diagnostics::UNKNOWN_COMPILER_OPTION_0_DID_YOU_MEAN_1),
        "watchOptions" => Some(diagnostics::UNKNOWN_WATCH_OPTION_0_DID_YOU_MEAN_1),
        "typeAcquisition" => Some(diagnostics::UNKNOWN_TYPE_ACQUISITION_OPTION_0_DID_YOU_MEAN_1),
        "buildOptions" => Some(diagnostics::UNKNOWN_BUILD_OPTION_0_DID_YOU_MEAN_1),
        _ => None,
    }
}

pub static LIB_MAP: &[(&str, &str)] = &[
    ("es5", "lib.es5.d.ts"),
    ("es6", "lib.es2015.d.ts"),
    ("es2015", "lib.es2015.d.ts"),
    ("es7", "lib.es2016.d.ts"),
    ("es2016", "lib.es2016.d.ts"),
    ("es2017", "lib.es2017.d.ts"),
    ("es2018", "lib.es2018.d.ts"),
    ("es2019", "lib.es2019.d.ts"),
    ("es2020", "lib.es2020.d.ts"),
    ("es2021", "lib.es2021.d.ts"),
    ("es2022", "lib.es2022.d.ts"),
    ("es2023", "lib.es2023.d.ts"),
    ("es2024", "lib.es2024.d.ts"),
    ("es2025", "lib.es2025.d.ts"),
    ("esnext", "lib.esnext.d.ts"),
    ("dom", "lib.dom.d.ts"),
    ("dom.iterable", "lib.dom.iterable.d.ts"),
    ("dom.asynciterable", "lib.dom.asynciterable.d.ts"),
    ("webworker", "lib.webworker.d.ts"),
    ("webworker.importscripts", "lib.webworker.importscripts.d.ts"),
    ("webworker.iterable", "lib.webworker.iterable.d.ts"),
    ("webworker.asynciterable", "lib.webworker.asynciterable.d.ts"),
    ("scripthost", "lib.scripthost.d.ts"),
];
