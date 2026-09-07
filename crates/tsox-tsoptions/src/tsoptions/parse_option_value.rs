#![allow(unused_imports)]

use super::*;

pub(crate) fn parse_option_value(
    args: &[String],
    mut i: usize,
    opt: &OptionDecl,
    inline_value: Option<String>,
    options: &mut HashMap<String, OptValue>,
    errors: &mut Vec<Diagnostic>,
    watch: bool,
) -> usize {
    let type_name = |kind: OptionKind| -> &'static str {
        match kind {
            OptionKind::Boolean => "boolean",
            OptionKind::String => "string",
            OptionKind::Number => "number",
            OptionKind::List => "list",
            OptionKind::Enum => "string",
        }
    };
    let missing_value_error = |errors: &mut Vec<Diagnostic>| {
        if watch {
            errors.push(Diagnostic::new(
                None,
                TextRange::undefined(),
                WATCH_OPTION_0_REQUIRES_A_VALUE_OF_TYPE_1,
                vec![opt.name.to_string(), type_name(opt.kind).to_string()],
            ));
        } else {
            errors.push(err(format!("Option '{}' requires a value.", opt.name)));
        }
    };

    if opt.is_tsconfig_only {
        let (opt_value, from_args) = match &inline_value {
            Some(v) => (v.clone(), false),
            None => {
                if i < args.len() {
                    (args[i].clone(), true)
                } else {
                    (String::new(), false)
                }
            }
        };
        if opt_value == "null" {
            options.insert(opt.name.to_string(), OptValue::Null);
            if from_args {
                i += 1;
            }
        } else if opt.kind == OptionKind::Boolean {
            if opt_value == "false" {
                options.insert(opt.name.to_string(), OptValue::Bool(false));
                if from_args {
                    i += 1;
                }
            } else {
                errors.push(Diagnostic::new(
                    None,
                    TextRange::undefined(),
                    OPTION_0_CAN_ONLY_BE_SPECIFIED_IN_TSCONFIG_JSON_FILE_OR_SET_TO_FALSE_OR_NULL_ON_COMMAND_LINE,
                    vec![opt.name.to_string()],
                ));
                if from_args && opt_value == "true" {
                    i += 1;
                }
            }
        } else {
            errors.push(Diagnostic::new(
                None,
                TextRange::undefined(),
                OPTION_0_CAN_ONLY_BE_SPECIFIED_IN_TSCONFIG_JSON_FILE_OR_SET_TO_NULL_ON_COMMAND_LINE,
                vec![opt.name.to_string()],
            ));
            if from_args && !opt_value.is_empty() && !opt_value.starts_with('-') {
                i += 1;
            }
        }
        return i;
    }

    match opt.kind {
        OptionKind::Boolean => {
            if let Some(v) = inline_value {
                let b = v != "false";
                options.insert(opt.name.to_string(), OptValue::Bool(b));
            } else if i < args.len() && (args[i] == "true" || args[i] == "false") {
                options.insert(opt.name.to_string(), OptValue::Bool(args[i] == "true"));
                i += 1;
            } else {
                options.insert(opt.name.to_string(), OptValue::Bool(true));
            }
        }
        OptionKind::String => {
            let val = match inline_value {
                Some(v) => Some(v),
                None => {
                    if i < args.len() {
                        let v = args[i].clone();
                        i += 1;
                        Some(v)
                    } else {
                        None
                    }
                }
            };
            match val {
                Some(v) if v == "null" => {
                    options.insert(opt.name.to_string(), OptValue::Null);
                }
                Some(v) => {
                    options.insert(opt.name.to_string(), OptValue::Str(v));
                }
                None => {
                    missing_value_error(errors);
                }
            }
        }
        OptionKind::Enum => {
            let val = match inline_value {
                Some(v) => Some(v),
                None => {
                    if i < args.len() {
                        let v = args[i].clone();
                        i += 1;
                        Some(v)
                    } else {
                        None
                    }
                }
            };
            match val {
                Some(v) if v == "null" => {
                    options.insert(opt.name.to_string(), OptValue::Null);
                }
                Some(v) => {
                    if let Some(enum_vals) = opt.enum_values {
                        if enum_vals.iter().any(|e| e.eq_ignore_ascii_case(&v)) {
                            options.insert(opt.name.to_string(), OptValue::Str(v));
                        } else {
                            let valid = enum_vals
                                .iter()
                                .map(|e| format!("'{}'", e))
                                .collect::<Vec<_>>()
                                .join(", ");
                            errors.push(Diagnostic::new(
                                None,
                                TextRange::undefined(),
                                ARGUMENT_FOR_0_OPTION_MUST_BE_COLON_1,
                                vec![format!("--{}", opt.name), valid],
                            ));
                        }
                    } else {
                        options.insert(opt.name.to_string(), OptValue::Str(v));
                    }
                }
                None => {
                    missing_value_error(errors);
                }
            }
        }
        OptionKind::Number => {
            let val = inline_value.or_else(|| {
                if i < args.len() {
                    let v = args[i].clone();
                    i += 1;
                    Some(v)
                } else {
                    None
                }
            });
            match val {
                Some(v) => match v.parse::<i64>() {
                    Ok(n) => {
                        if let Some(min) = opt.min_value {
                            if n < min {
                                errors.push(Diagnostic::new(
                                    None,
                                    TextRange::undefined(),
                                    OPTION_0_REQUIRES_VALUE_TO_BE_GREATER_THAN_1,
                                    vec![opt.name.to_string(), min.to_string()],
                                ));
                            } else {
                                options.insert(opt.name.to_string(), OptValue::Num(n));
                            }
                        } else {
                            options.insert(opt.name.to_string(), OptValue::Num(n));
                        }
                    }
                    Err(_) => {
                        if watch {
                            errors.push(Diagnostic::new(
                                None,
                                TextRange::undefined(),
                                WATCH_OPTION_0_REQUIRES_A_VALUE_OF_TYPE_1,
                                vec![opt.name.to_string(), type_name(opt.kind).to_string()],
                            ));
                        } else {
                            errors.push(err(format!("Option '{}' requires a number.", opt.name)));
                        }
                    }
                },
                None => {
                    missing_value_error(errors);
                }
            }
        }
        OptionKind::List => {
            let val = inline_value.or_else(|| {
                if i < args.len() && !args[i].starts_with('-') {
                    let v = args[i].clone();
                    i += 1;
                    Some(v)
                } else {
                    None
                }
            });
            let list = match val {
                Some(v) => v.split(',').map(|s| s.trim().to_string()).collect(),
                None => Vec::new(),
            };
            options.insert(opt.name.to_string(), OptValue::List(list));
        }
    }
    i
}

pub(crate) fn split_response_file(
    content: &str,
    file_name: &str,
) -> (Vec<String>, Vec<Diagnostic>) {
    let mut args = Vec::new();
    let mut errors: Vec<Diagnostic> = Vec::new();
    let chars: Vec<char> = content.chars().collect();
    let mut pos = 0usize;
    while pos < chars.len() {
        while pos < chars.len() && chars[pos] <= ' ' {
            pos += 1;
        }
        if pos >= chars.len() {
            break;
        }
        if chars[pos] == '"' {
            pos += 1;
            let start = pos;
            while pos < chars.len() && chars[pos] != '"' {
                pos += 1;
            }
            args.push(chars[start..pos].iter().collect());
            if pos < chars.len() {
                pos += 1;
            } else {
                errors.push(Diagnostic::new(
                    None,
                    TextRange::undefined(),
                    UNTERMINATED_QUOTED_STRING_IN_RESPONSE_FILE_0,
                    vec![file_name.to_string()],
                ));
            }
        } else {
            let start = pos;
            while pos < chars.len() && chars[pos] > ' ' {
                pos += 1;
            }
            args.push(chars[start..pos].iter().collect());
        }
    }
    (args, errors)
}
