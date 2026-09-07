#![allow(unused_imports)]

use super::*;

pub(crate) fn parse_command_line_worker(
    args: &[String],
    current_dir: &str,
    fs: Option<&dyn FS>,
    find: fn(&str) -> Option<&'static OptionDecl>,
    mode: ParseMode,
) -> (
    HashMap<String, OptValue>,
    HashMap<String, OptValue>,
    Vec<String>,
    Vec<Diagnostic>,
) {
    let mut options: HashMap<String, OptValue> = HashMap::new();
    let mut watch_options: HashMap<String, OptValue> = HashMap::new();
    let mut file_names: Vec<String> = Vec::new();
    let mut errors: Vec<Diagnostic> = Vec::new();

    let mut i = 0usize;
    while i < args.len() {
        let s = &args[i];
        i += 1;
        if s.is_empty() {
            continue;
        }
        let first = s.chars().next().unwrap();
        match first {
            '@' => {
                let response_path = &s[1..];
                let abs = tspath::get_normalized_absolute_path(response_path, current_dir);
                if let Some(fs) = fs {
                    if let Some(content) = fs.read_file(&abs) {
                        let (response_args, split_errors) = split_response_file(&content, &abs);
                        errors.extend(split_errors);
                        let (sub_options, sub_watch_options, sub_files, sub_errors) =
                            parse_command_line_worker(
                                &response_args,
                                current_dir,
                                Some(fs),
                                find,
                                mode,
                            );
                        file_names.extend(sub_files);
                        for (k, v) in sub_options {
                            options.insert(k, v);
                        }
                        for (k, v) in sub_watch_options {
                            watch_options.insert(k, v);
                        }
                        errors.extend(sub_errors);
                    } else {
                        errors.push(Diagnostic::new(
                            None,
                            TextRange::undefined(),
                            CANNOT_READ_FILE_0,
                            vec![response_path.to_string()],
                        ));
                    }
                } else {
                    errors.push(Diagnostic::new(
                        None,
                        TextRange::undefined(),
                        CANNOT_READ_FILE_0,
                        vec![response_path.to_string()],
                    ));
                }
            }
            '-' => {
                let name_part = s.trim_start_matches('-');

                let (name, inline_value) = match name_part.split_once('=') {
                    Some((n, v)) => (n, Some(v.to_string())),
                    None => (name_part, None),
                };
                match find(name) {
                    Some(opt) => {
                        i = parse_option_value(
                            args,
                            i,
                            opt,
                            inline_value,
                            &mut options,
                            &mut errors,
                            false,
                        );
                    }
                    None => {
                        if let Some(opt) = find_watch_option(name) {
                            i = parse_option_value(
                                args,
                                i,
                                opt,
                                inline_value,
                                &mut watch_options,
                                &mut errors,
                                true,
                            );
                            continue;
                        }

                        if mode == ParseMode::Compiler && find_build_only_option(name).is_some() {
                            errors.push(Diagnostic::new(
                                None,
                                TextRange::undefined(),
                                COMPILER_OPTION_0_MAY_ONLY_BE_USED_WITH_BUILD,
                                vec![name.to_string()],
                            ));
                            continue;
                        }

                        if mode == ParseMode::Build {
                            if find_option(name).is_some() {
                                errors.push(Diagnostic::new(
                                    None,
                                    TextRange::undefined(),
                                    COMPILER_OPTION_0_MAY_NOT_BE_USED_WITH_BUILD,
                                    vec![name.to_string()],
                                ));
                                continue;
                            }

                            let suggestion = did_you_mean_build_option(name);
                            if let Some(s) = suggestion {
                                errors.push(Diagnostic::new(
                                    None,
                                    TextRange::undefined(),
                                    UNKNOWN_BUILD_OPTION_0_DID_YOU_MEAN_1,
                                    vec![name.to_string(), s],
                                ));
                            } else {
                                errors.push(Diagnostic::new(
                                    None,
                                    TextRange::undefined(),
                                    UNKNOWN_BUILD_OPTION_0,
                                    vec![name.to_string()],
                                ));
                            }
                            continue;
                        }
                        errors.push(Diagnostic::new(
                            None,
                            TextRange::undefined(),
                            UNKNOWN_COMPILER_OPTION_0,
                            vec![name.to_string()],
                        ));
                        continue;
                    }
                }
            }
            _ => {
                file_names.push(s.clone());
            }
        }
    }
    (options, watch_options, file_names, errors)
}
