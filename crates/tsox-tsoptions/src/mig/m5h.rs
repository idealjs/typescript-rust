#![allow(dead_code, unused_imports, unused_variables)]

use std::sync::Arc;

use tsox_core::collections::ordered_map::OrderedMap;
use tsox_core::collections::set::Set;
use tsox_core::diagnostics::Message;
use tsox_core::diagnostics::{
    CANNOT_READ_FILE_0,
    COMPILER_OPTION_0_EXPECTS_AN_ARGUMENT,
    COMPILER_OPTION_0_MAY_ONLY_BE_USED_WITH_BUILD,
    OPTION_0_CAN_ONLY_BE_SPECIFIED_IN_TSCONFIG_JSON_FILE_OR_SET_TO_FALSE_OR_NULL_ON_COMMAND_LINE,
    OPTION_0_CAN_ONLY_BE_SPECIFIED_IN_TSCONFIG_JSON_FILE_OR_SET_TO_NULL_ON_COMMAND_LINE,
    OPTION_0_REQUIRES_VALUE_TO_BE_GREATER_THAN_1,
    UNKNOWN_COMPILER_OPTION_0,
    UNKNOWN_COMPILER_OPTION_0_DID_YOU_MEAN_1,
    UNKNOWN_WATCH_OPTION_0,
    UNKNOWN_WATCH_OPTION_0_DID_YOU_MEAN_1,
    UNTERMINATED_QUOTED_STRING_IN_RESPONSE_FILE_0,
    WATCH_OPTION_0_REQUIRES_A_VALUE_OF_TYPE_1,
};
use tsox_core::json::Value as JsonValue;
use tsox_core::tspath::Path;
use tsox_frontend::ast::{Diagnostic, Node, SourceFile};

use super::m5h_2::{get_name_map_from_list, NameMap};
use super::m5h_4::{create_diagnostic_for_invalid_enum_type, get_compiler_option_value_type_string};
use super::m5j::validate_json_option_value;
use tsox_frontend::ast::mig::m3d_2::new_compiler_diagnostic;
use crate::tsoptions::build_options::{OptValue, OPTIONS_FOR_WATCH};
use crate::tsoptions::option_kind::{OptionDecl, OptionKind};
use crate::tsoptions::options_options::OPTIONS;
use crate::vfs::FS;

pub struct AlternateModeDiagnostics {
    pub diagnostic: Message,
    pub options_name_map: NameMap,
}

pub struct DidYouMeanOptionsDiagnostics {
    pub alternate_mode: Option<AlternateModeDiagnostics>,
    pub option_declarations: Vec<OptionDecl>,
    pub unknown_option_diagnostic: Message,
    pub unknown_did_you_mean_diagnostic: Message,
}

pub struct ParseCommandLineWorkerDiagnostics {
    pub did_you_mean: DidYouMeanOptionsDiagnostics,
    pub options_name_map: Option<NameMap>,
    pub option_type_mismatch_diagnostic: Message,
}

const COMMON_OPTIONS_WITH_BUILD: &[&str] = &[
    "help",
    "watch",
    "preserveWatchOutput",
    "listFiles",
    "explainFiles",
    "listEmittedFiles",
    "pretty",
    "traceResolution",
    "diagnostics",
    "extendedDiagnostics",
    "generateCpuProfile",
    "generateTrace",
    "incremental",
    "declaration",
    "declarationMap",
    "emitDeclarationOnly",
    "sourceMap",
    "inlineSourceMap",
    "noCheck",
    "deduplicatePackages",
    "noEmit",
    "assumeChangesOnlyAffectDirectDependencies",
    "locale",
    "quiet",
    "singleThreaded",
    "pprofDir",
    "checkers",
    "runExternalCode",
];

pub fn build_name_map() -> NameMap {
    let mut decls: Vec<OptionDecl> = OPTIONS
        .iter()
        .filter(|o| COMMON_OPTIONS_WITH_BUILD.contains(&o.name))
        .copied()
        .collect();
    decls.extend(crate::tsoptions::build_options::BUILD_OPTIONS.iter().copied());
    get_name_map_from_list(&decls)
}

pub fn watch_name_map() -> NameMap {
    get_name_map_from_list(OPTIONS_FOR_WATCH)
}

pub fn watch_options_did_you_mean_diagnostics() -> ParseCommandLineWorkerDiagnostics {
    ParseCommandLineWorkerDiagnostics {
        did_you_mean: DidYouMeanOptionsDiagnostics {
            alternate_mode: None,
            option_declarations: OPTIONS_FOR_WATCH.to_vec(),
            unknown_option_diagnostic: UNKNOWN_WATCH_OPTION_0,
            unknown_did_you_mean_diagnostic: UNKNOWN_WATCH_OPTION_0_DID_YOU_MEAN_1,
        },
        options_name_map: None,
        option_type_mismatch_diagnostic: WATCH_OPTION_0_REQUIRES_A_VALUE_OF_TYPE_1,
    }
}

pub fn get_parse_command_line_worker_diagnostics(
    decls: Vec<OptionDecl>,
) -> ParseCommandLineWorkerDiagnostics {
    ParseCommandLineWorkerDiagnostics {
        did_you_mean: DidYouMeanOptionsDiagnostics {
            alternate_mode: Some(AlternateModeDiagnostics {
                diagnostic: COMPILER_OPTION_0_MAY_ONLY_BE_USED_WITH_BUILD,
                options_name_map: build_name_map(),
            }),
            option_declarations: decls,
            unknown_option_diagnostic: UNKNOWN_COMPILER_OPTION_0,
            unknown_did_you_mean_diagnostic: UNKNOWN_COMPILER_OPTION_0_DID_YOU_MEAN_1,
        },
        options_name_map: None,
        option_type_mismatch_diagnostic: COMPILER_OPTION_0_EXPECTS_AN_ARGUMENT,
    }
}

pub struct CommandLineParser {
    pub worker_diagnostics: ParseCommandLineWorkerDiagnostics,
    pub options_map: NameMap,
    pub fs: Option<Arc<dyn FS>>,
    pub current_directory: String,
    pub options: OrderedMap<String, OptValue>,
    pub file_names: Vec<String>,
    pub errors: Vec<Diagnostic>,
    pub response_file_stack: Set<Path>,
}

impl CommandLineParser {
    pub fn alternate_mode(&self) -> Option<&AlternateModeDiagnostics> {
        self.worker_diagnostics.did_you_mean.alternate_mode.as_ref()
    }

    pub fn options_declarations(&self) -> &[OptionDecl] {
        &self.worker_diagnostics.did_you_mean.option_declarations
    }

    pub fn unknown_option_diagnostic(&self) -> Message {
        self.worker_diagnostics.did_you_mean.unknown_option_diagnostic
    }

    pub fn unknown_did_you_mean_diagnostic(&self) -> Message {
        self.worker_diagnostics
            .did_you_mean
            .unknown_did_you_mean_diagnostic
    }

    pub fn parse_strings(&mut self, args: &[String]) {
        let mut i = 0;
        while i < args.len() {
            let s = &args[i];
            i += 1;
            if s.is_empty() {
                continue;
            }
            match s.as_bytes()[0] {
                b'@' => self.parse_response_file(&s[1..]),
                b'-' => {
                    let input_option_name = get_input_option_name(s);
                    let opt = self
                        .options_map
                        .get_option_declaration_from_name(&input_option_name, true)
                        .copied();
                    let mismatch = self.worker_diagnostics.option_type_mismatch_diagnostic;
                    if let Some(opt) = opt {
                        i = self.parse_option_value(args, i, &opt, mismatch);
                    } else {
                        let watch_opt = watch_name_map()
                            .get_option_declaration_from_name(&input_option_name, true)
                            .copied();
                        if let Some(watch_opt) = watch_opt {
                            let watch_mismatch = watch_options_did_you_mean_diagnostics()
                                .option_type_mismatch_diagnostic;
                            i = self.parse_option_value(args, i, &watch_opt, watch_mismatch);
                        } else {
                            let e = self.create_unknown_option_error(
                                &input_option_name,
                                s,
                                None,
                                None,
                            );
                            self.errors.push(e);
                        }
                    }
                }
                _ => self.file_names.push(s.clone()),
            }
        }
    }

    pub fn parse_option_value(
        &mut self,
        args: &[String],
        mut i: usize,
        opt: &OptionDecl,
        diag: Message,
    ) -> usize {
        if opt.is_tsconfig_only && i <= args.len() {
            let opt_value = if i < args.len() {
                args[i].clone()
            } else {
                String::new()
            };
            if opt_value == "null" {
                self.options.set(opt.name.to_string(), OptValue::Null);
                i += 1;
            } else if opt.kind == OptionKind::Boolean {
                if opt_value == "false" {
                    self.options.set(opt.name.to_string(), OptValue::Bool(false));
                    i += 1;
                } else {
                    if opt_value == "true" {
                        i += 1;
                    }
                    self.errors.push(new_compiler_diagnostic(
                        OPTION_0_CAN_ONLY_BE_SPECIFIED_IN_TSCONFIG_JSON_FILE_OR_SET_TO_FALSE_OR_NULL_ON_COMMAND_LINE,
                        vec![opt.name.to_string()],
                    ));
                }
            } else {
                self.errors.push(new_compiler_diagnostic(
                    OPTION_0_CAN_ONLY_BE_SPECIFIED_IN_TSCONFIG_JSON_FILE_OR_SET_TO_NULL_ON_COMMAND_LINE,
                    vec![opt.name.to_string()],
                ));
                if !opt_value.is_empty() && !opt_value.starts_with('-') {
                    i += 1;
                }
            }
        } else {
            if i >= args.len() {
                if opt.kind != OptionKind::Boolean {
                    self.errors.push(new_compiler_diagnostic(
                        diag,
                        vec![opt.name.to_string(), get_compiler_option_value_type_string(opt)],
                    ));
                    if opt.kind == OptionKind::List {
                        self.options
                            .set(opt.name.to_string(), OptValue::List(Vec::new()));
                    } else if opt.kind == OptionKind::Enum {
                        self.errors
                            .push(create_diagnostic_for_invalid_enum_type(opt, None, None));
                    }
                } else {
                    self.options.set(opt.name.to_string(), OptValue::Bool(true));
                }
                return i;
            }
            if args[i] != "null" {
                match opt.kind {
                    OptionKind::Number => match args[i].parse::<i64>() {
                        Ok(num) => {
                            if num >= opt.min_value.unwrap_or(0) {
                                self.options.set(opt.name.to_string(), OptValue::Num(num));
                            } else {
                                self.errors.push(new_compiler_diagnostic(
                                    OPTION_0_REQUIRES_VALUE_TO_BE_GREATER_THAN_1,
                                    vec![
                                        opt.name.to_string(),
                                        opt.min_value.unwrap_or(0).to_string(),
                                    ],
                                ));
                            }
                        }
                        Err(_) => {
                            self.errors.push(new_compiler_diagnostic(
                                diag,
                                vec![opt.name.to_string(), "number".to_string()],
                            ));
                        }
                    },
                    OptionKind::Boolean => {
                        let opt_value = &args[i];
                        self.options.set(
                            opt.name.to_string(),
                            OptValue::Bool(opt_value != "false"),
                        );
                        if opt_value == "false" || opt_value == "true" {
                            i += 1;
                        }
                    }
                    OptionKind::String => {
                        let (val, errors) = validate_json_option_value(
                            opt,
                            &JsonValue::String(args[i].clone()),
                            None,
                            None,
                        );
                        if errors.is_empty() {
                            self.options
                                .set(opt.name.to_string(), json_to_opt_value(val.as_ref()));
                        } else {
                            self.errors.extend(errors);
                        }
                        i += 1;
                    }
                    OptionKind::List | OptionKind::ListOrElement => {
                        let (result, errors) = parse_list_type_option(opt, &args[i]);
                        let strings: Vec<String> = result
                            .iter()
                            .filter_map(|v| v.as_str().map(String::from))
                            .collect();
                        self.options
                            .set(opt.name.to_string(), OptValue::List(strings));
                        let consumed = !result.is_empty() || !errors.is_empty();
                        self.errors.extend(errors);
                        if consumed {
                            i += 1;
                        }
                    }
                    OptionKind::Enum => {
                        let trimmed = args[i]
                            .trim_matches(|c: char| tsox_core::stringutil::is_white_space_like(c));
                        let (val, errors) =
                            convert_json_option_of_enum_type(opt, trimmed, None, None);
                        self.options
                            .set(opt.name.to_string(), val.unwrap_or(OptValue::Null));
                        self.errors.extend(errors);
                        i += 1;
                    }
                }
            } else {
                self.options.set(opt.name.to_string(), OptValue::Null);
                i += 1;
            }
        }
        i
    }
}

fn json_to_opt_value(value: Option<&JsonValue>) -> OptValue {
    match value {
        Some(JsonValue::Bool(b)) => OptValue::Bool(*b),
        Some(JsonValue::Number(n)) => OptValue::Num(n.as_i64().unwrap_or(0)),
        Some(JsonValue::String(s)) => OptValue::Str(s.clone()),
        Some(JsonValue::Array(items)) => OptValue::List(
            items
                .iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect(),
        ),
        _ => OptValue::Null,
    }
}

fn opt_value_to_json(value: &OptValue) -> JsonValue {
    match value {
        OptValue::Bool(b) => JsonValue::Bool(*b),
        OptValue::Num(n) => JsonValue::Number((*n).into()),
        OptValue::Str(s) => JsonValue::String(s.clone()),
        OptValue::List(items) => {
            JsonValue::Array(items.iter().map(|s| JsonValue::String(s.clone())).collect())
        }
        OptValue::Null => JsonValue::Null,
    }
}

pub fn get_input_option_name(input: &str) -> String {
    let mut s = input;
    if let Some(rest) = s.strip_prefix('-') {
        s = rest;
    }
    if let Some(rest) = s.strip_prefix('-') {
        s = rest;
    }
    s.to_string()
}

pub fn try_read_file(
    file_name: &str,
    read_file: &dyn Fn(&str) -> Option<String>,
    errors: &mut Vec<Diagnostic>,
) -> String {
    match read_file(file_name) {
        Some(text) => text,
        None => {
            errors.push(new_compiler_diagnostic(
                CANNOT_READ_FILE_0,
                vec![file_name.to_string()],
            ));
            String::new()
        }
    }
}

impl CommandLineParser {
    pub fn parse_response_file(&mut self, file_name: &str) {
        let file_name =
            tsox_core::tspath::get_normalized_absolute_path(file_name, &self.current_directory);
        let path = tsox_core::tspath::to_path(
            &file_name,
            &self.current_directory,
            self.fs.as_ref().map(|f| f.use_case_sensitive_file_names()).unwrap_or(true),
        );
        if self.response_file_stack.has(&path) {
            return;
        }
        self.response_file_stack.add(path.clone());

        let fs = self.fs.clone();
        let read_file = move |name: &str| -> Option<String> {
            fs.as_ref().and_then(|f| f.read_file(name))
        };
        let contents = try_read_file(&file_name, &read_file, &mut self.errors);
        if contents.is_empty() {
            self.response_file_stack.delete(&path);
            return;
        }
        let text: Vec<char> = contents.chars().collect();
        let text_length = text.len();
        let mut pos = 0;
        let mut args: Vec<String> = Vec::new();
        while pos < text_length {
            while pos < text_length && text[pos] <= ' ' {
                pos += 1;
            }
            if pos >= text_length {
                break;
            }
            let start = pos;
            if text[pos] == '"' {
                pos += 1;
                while pos < text_length && text[pos] != '"' {
                    pos += 1;
                }
                if pos < text_length {
                    args.push(text[start + 1..pos].iter().collect());
                    pos += 1;
                } else {
                    self.errors.push(new_compiler_diagnostic(
                        UNTERMINATED_QUOTED_STRING_IN_RESPONSE_FILE_0,
                        vec![file_name.clone()],
                    ));
                }
            } else {
                while pos < text_length && text[pos] > ' ' {
                    pos += 1;
                }
                args.push(text[start..pos].iter().collect());
            }
        }
        self.response_file_stack.delete(&path);
        self.parse_strings(&args);
    }
}

pub fn convert_json_option_of_enum_type(
    opt: &OptionDecl,
    value: &str,
    value_expression: Option<&Node>,
    source_file: Option<&SourceFile>,
) -> (Option<OptValue>, Vec<Diagnostic>) {
    if value.is_empty() {
        return (None, Vec::new());
    }
    let key = value.to_lowercase();
    let type_map = match opt.enum_map() {
        Some(m) => m,
        None => return (None, Vec::new()),
    };
    match type_map.get(&key) {
        Some(val) => {
            let (val, errors) = validate_json_option_value(
                opt,
                &opt_value_to_json(val),
                value_expression,
                source_file,
            );
            (Some(json_to_opt_value(val.as_ref())), errors)
        }
        None => (
            None,
            vec![create_diagnostic_for_invalid_enum_type(
                opt,
                source_file,
                value_expression,
            )],
        ),
    }
}

pub fn parse_list_type_option(
    opt: &OptionDecl,
    value: &str,
) -> (Vec<OptValue>, Vec<Diagnostic>) {
    let value = value.trim();
    let mut errors: Vec<Diagnostic> = Vec::new();
    if value.starts_with('-') {
        return (Vec::new(), errors);
    }
    if opt.kind == OptionKind::ListOrElement && !value.contains(',') {
        let (val, err) = validate_json_option_value(
            opt,
            &JsonValue::String(value.to_string()),
            None,
            None,
        );
        if !err.is_empty() {
            return (Vec::new(), err);
        }
        let single = val
            .as_ref()
            .and_then(|v| v.as_str().map(|s| OptValue::Str(s.to_string())))
            .unwrap_or(OptValue::Null);
        return (vec![single], errors);
    }
    if value.is_empty() {
        return (Vec::new(), errors);
    }
    let values: Vec<&str> = value.split(',').collect();
    match opt.elements().map(|e| e.kind) {
        Some(OptionKind::String) => {
            let mut elements: Vec<OptValue> = Vec::new();
            for v in values {
                let (val, err) = validate_json_option_value(
                    opt.elements().unwrap(),
                    &JsonValue::String(v.to_string()),
                    None,
                    None,
                );
                let keep = match &val {
                    Some(s) if s.is_string() && !s.as_str().unwrap_or("").is_empty() => {
                        err.is_empty()
                    }
                    _ => false,
                };
                errors.extend(err);
                if keep {
                    elements.push(json_to_opt_value(val.as_ref()));
                }
            }
            (elements, errors)
        }
        Some(OptionKind::Boolean) | Some(OptionKind::Number) => {
            panic!("List of boolean/object/number is not yet supported.")
        }
        _ => {
            let Some(element_decl) = opt.elements() else {
                return (Vec::new(), errors);
            };
            let mut result: Vec<OptValue> = Vec::new();
            for v in values {
                let trimmed =
                    v.trim_matches(|c: char| tsox_core::stringutil::is_white_space_like(c));
                let (val, err) = convert_json_option_of_enum_type(element_decl, trimmed, None, None);
                let keep = match &val {
                    Some(OptValue::Str(s)) => !s.is_empty() && err.is_empty(),
                    _ => false,
                };
                errors.extend(err);
                if keep {
                    result.push(val.unwrap());
                }
            }
            (result, errors)
        }
    }
}
