use super::*;
use crate::vfs::InMemoryFS;

#[test]
fn parse_basic_options() {
    let args: Vec<String> = vec!["--noEmit", "--strict", "--target", "ES2020", "src/a.ts"]
        .into_iter()
        .map(String::from)
        .collect();
    let parsed = parse_command_line(&args, "/proj", None);
    assert!(parsed.compiler_options.no_emit.is_true());
    assert!(parsed.compiler_options.strict.is_true());
    assert!(parsed.compiler_options.strict_null_checks.is_true());
    assert_eq!(parsed.compiler_options.target, ScriptTarget::ES2020);
    assert_eq!(parsed.file_names, vec!["/proj/src/a.ts"]);
}

#[test]
fn parse_equals_form() {
    let args: Vec<String> = vec!["--target=ES2015", "--module=commonjs"]
        .into_iter()
        .map(String::from)
        .collect();
    let parsed = parse_command_line(&args, "/proj", None);
    assert_eq!(parsed.compiler_options.target, ScriptTarget::ES2015);
    assert_eq!(parsed.compiler_options.module, ModuleKind::CommonJS);
}

#[test]
fn parse_short_option() {
    let args: Vec<String> = vec!["-p", "tsconfig.json"]
        .into_iter()
        .map(String::from)
        .collect();
    let parsed = parse_command_line(&args, "/proj", None);
    assert_eq!(parsed.compiler_options.project, "tsconfig.json");
}

#[test]
fn strip_jsonc_comments() {
    let input = r#"{ // comment
            "compilerOptions": {
                "target": "ES5", /* block */
                "strict": true,
            }
        }"#;
    let stripped = strip_jsonc(input);
    let v: crate::json::Value = crate::json::from_str(&stripped).unwrap();
    assert_eq!(v["compilerOptions"]["target"].as_str(), Some("ES5"));
}

#[test]
fn parse_tsconfig_files() {
    let fs = InMemoryFS::new();
    fs.insert_dir("/proj");
    fs.insert_dir("/proj/src");
    fs.insert_file(
        "/proj/tsconfig.json",
        r#"{
            "compilerOptions": { "target": "ES2017", "noEmit": true },
            "files": ["src/a.ts"]
        }"#,
    );
    fs.insert_file("/proj/src/a.ts", "export const x = 1;");
    let parsed = get_parsed_command_line_of_config_file(
        "/proj/tsconfig.json",
        &CompilerOptions::default(),
        "/proj",
        &fs,
    );
    assert_eq!(parsed.compiler_options.target, ScriptTarget::ES2017);
    assert!(parsed.compiler_options.no_emit.is_true());
    assert_eq!(parsed.file_names, vec!["/proj/src/a.ts"]);
}

#[test]
fn parse_tsconfig_include_glob() {
    let fs = InMemoryFS::new();
    fs.insert_dir("/proj");
    fs.insert_dir("/proj/src");
    fs.insert_file(
        "/proj/tsconfig.json",
        r#"{
            "include": ["src/**/*"]
        }"#,
    );
    fs.insert_file("/proj/src/a.ts", "export const a = 1;");
    fs.insert_file("/proj/src/b.ts", "export const b = 2;");
    fs.insert_file("/proj/src/ignore.txt", "ignore me");
    let parsed = get_parsed_command_line_of_config_file(
        "/proj/tsconfig.json",
        &CompilerOptions::default(),
        "/proj",
        &fs,
    );
    assert!(parsed.file_names.contains(&"/proj/src/a.ts".to_string()));
    assert!(parsed.file_names.contains(&"/proj/src/b.ts".to_string()));
    assert!(!parsed.file_names.iter().any(|f| f.ends_with("ignore.txt")));
}

fn has_error_containing(parsed: &ParsedCommandLine, needle: &str) -> bool {
    parsed.errors.iter().any(|e| {
        e.message_args.iter().any(|a| a.contains(needle))
            || e.message.map(|m| m.text.contains(needle)).unwrap_or(false)
    })
}

fn args(items: &[&str]) -> Vec<String> {
    items.iter().map(|s| s.to_string()).collect()
}

#[test]
fn test_parse_command_line_version() {
    let parsed = parse_command_line(&args(&["--version"]), "/proj", None);
    assert!(parsed.compiler_options.version.is_true());

    let parsed_short = parse_command_line(&args(&["-v"]), "/proj", None);
    assert!(parsed_short.compiler_options.version.is_true());
}

#[test]
fn test_parse_command_line_help() {
    let parsed = parse_command_line(&args(&["--help"]), "/proj", None);
    assert!(parsed.compiler_options.help.is_true());

    let parsed_short = parse_command_line(&args(&["-h"]), "/proj", None);
    assert!(parsed_short.compiler_options.help.is_true());
}

#[test]
fn test_parse_command_line_build() {
    let parsed = parse_command_line(&args(&["--build"]), "/proj", None);
    assert!(parsed.compiler_options.build.is_true());

    let parsed_short = parse_command_line(&args(&["-b"]), "/proj", None);
    assert!(parsed_short.compiler_options.build.is_true());
}

#[test]
fn test_parse_build_command_line_defaults_to_current_project() {
    let parsed = parse_build_command_line(&args(&["--build"]), "/proj", None);
    assert_eq!(parsed.projects, vec!["."]);
    assert_eq!(parsed.resolved_project_paths(), vec!["/proj"]);
    assert!(parsed.compiler_options.build.is_true());
}

#[test]
fn test_parse_build_command_line_build_options() {
    let parsed = parse_build_command_line(
        &args(&["--build", "src", "tests", "--force", "-v", "--dry"]),
        "/repo",
        None,
    );
    assert_eq!(parsed.projects, vec!["src", "tests"]);
    assert_eq!(
        parsed.resolved_project_paths(),
        vec!["/repo/src", "/repo/tests"]
    );
    assert!(parsed.build_options.force.is_true());
    assert!(parsed.build_options.verbose.is_true());
    assert!(parsed.build_options.dry.is_true());
    assert!(parsed.compiler_options.build.is_true());
    assert!(!parsed.compiler_options.version.is_true());
}

#[test]
fn test_parse_build_command_line_invalid_option_combinations() {
    let parsed = parse_build_command_line(&args(&["--build", "--clean", "--force"]), "/proj", None);
    assert!(has_error_containing(
        &ParsedCommandLine {
            errors: parsed.errors,
            ..ParsedCommandLine::default()
        },
        "cannot be combined"
    ));

    let parsed = parse_build_command_line(&args(&["--build", "--watch", "--dry"]), "/proj", None);
    assert!(has_error_containing(
        &ParsedCommandLine {
            errors: parsed.errors,
            ..ParsedCommandLine::default()
        },
        "cannot be combined"
    ));
}

#[test]
fn test_parse_command_line_watch() {
    let parsed = parse_command_line(&args(&["--watch", "0.ts"]), "/proj", None);
    assert!(parsed.compiler_options.watch.is_true());

    assert!(parsed.watch);

    let parsed_short = parse_command_line(&args(&["-w", "0.ts"]), "/proj", None);
    assert!(parsed_short.compiler_options.watch.is_true());
    assert!(parsed_short.watch);
}

#[test]
fn watch_options_empty_by_default() {
    let parsed = parse_command_line(&args(&["--noEmit", "0.ts"]), "/proj", None);
    assert!(parsed.watch_options.is_empty());
}

#[test]
fn watch_options_parse_enum_flags() {
    let parsed = parse_command_line(
        &args(&[
            "--watchFile",
            "UseFsEvents",
            "--watchDirectory",
            "fixedpollinginterval",
            "--fallbackPolling",
            "priorityinterval",
            "0.ts",
        ]),
        "/proj",
        None,
    );
    assert_eq!(parsed.watch_options.file_kind, WatchFileKind::UseFsEvents);
    assert_eq!(
        parsed.watch_options.directory_kind,
        WatchDirectoryKind::FixedPollingInterval
    );
    assert_eq!(
        parsed.watch_options.fallback_polling,
        PollingKind::PriorityInterval
    );
}

#[test]
fn watch_options_parse_interval_and_boolean() {
    let parsed = parse_command_line(
        &args(&[
            "--watchInterval",
            "250",
            "--synchronousWatchDirectory",
            "0.ts",
        ]),
        "/proj",
        None,
    );
    assert_eq!(parsed.watch_options.interval, Some(250));
    assert_eq!(parsed.watch_options.watch_interval_ms(), 250);
    assert!(parsed.watch_options.sync_watch_dir.is_true());
}

#[test]
fn watch_options_parse_list_flags() {
    let parsed = parse_command_line(
        &args(&[
            "--excludeDirectories",
            "tmp,build",
            "--excludeFiles",
            "a.ts,b.ts",
            "0.ts",
        ]),
        "/proj",
        None,
    );
    assert_eq!(parsed.watch_options.exclude_dir, vec!["tmp", "build"]);
    assert_eq!(parsed.watch_options.exclude_files, vec!["a.ts", "b.ts"]);
}

#[test]
fn watch_options_invalid_enum_reports_ts6046() {
    let parsed = parse_command_line(&args(&["--watchFile", "bogus", "0.ts"]), "/proj", None);
    assert!(
        parsed
            .errors
            .iter()
            .any(|d| d.code == 6046 && d.message_args.iter().any(|a| a.contains("--watchFile")))
    );

    assert_eq!(parsed.watch_options.file_kind, WatchFileKind::None);
}

#[test]
fn watch_options_missing_number_value_reports_ts5080() {
    let parsed = parse_command_line(&args(&["--watchInterval"]), "/proj", None);
    assert!(parsed.errors.iter().any(|d| d.code == 5080
        && d.message_args.first().map(|s| s.as_str()) == Some("watchInterval")
        && d.message_args.get(1).map(|s| s.as_str()) == Some("number")));
}

#[test]
fn watch_options_non_numeric_interval_reports_ts5080() {
    let parsed = parse_command_line(&args(&["--watchInterval", "abc", "0.ts"]), "/proj", None);
    assert!(parsed.errors.iter().any(|d| d.code == 5080));
    assert_eq!(parsed.watch_options.interval, None);
}

#[test]
fn watch_options_build_mode_also_accepts_watch_flags() {
    let parsed = parse_build_command_line(
        &args(&["--build", "--watchFile", "usefsevents", "."]),
        "/proj",
        None,
    );
    assert_eq!(parsed.watch_options.file_kind, WatchFileKind::UseFsEvents);
}

#[test]
fn watch_options_case_insensitive_lookup() {
    let parsed = parse_command_line(
        &args(&["--WATCHFILE", "usefsevents", "0.ts"]),
        "/proj",
        None,
    );
    assert_eq!(parsed.watch_options.file_kind, WatchFileKind::UseFsEvents);
}

#[test]
fn watch_options_do_not_leak_into_compiler_options() {
    let parsed = parse_command_line(
        &args(&["--watchFile", "usefsevents", "0.ts"]),
        "/proj",
        None,
    );
    assert!(
        !parsed
            .errors
            .iter()
            .any(|d| d.code == 5023 && d.message_args.iter().any(|a| a == "watchFile"))
    );
    assert_eq!(parsed.watch_options.file_kind, WatchFileKind::UseFsEvents);
}

#[test]
fn test_parse_command_line_all_and_init() {
    let parsed = parse_command_line(&args(&["--all"]), "/proj", None);
    assert!(parsed.compiler_options.all.is_true());

    let parsed = parse_command_line(&args(&["--init"]), "/proj", None);
    assert!(parsed.compiler_options.init.is_true());
}

#[test]
fn test_parse_command_line_lib_list() {
    let parsed = parse_command_line(
        &args(&["--lib", "es5,es2015.symbol.wellknown", "0.ts"]),
        "/proj",
        None,
    );
    assert_eq!(
        parsed.compiler_options.lib,
        vec!["es5".to_string(), "es2015.symbol.wellknown".to_string()]
    );
    assert_eq!(parsed.file_names, vec!["/proj/0.ts"]);
}

#[test]
fn test_parse_command_line_lib_multiple_flags() {
    let parsed = parse_command_line(
        &args(&[
            "--module",
            "commonjs",
            "--target",
            "es5",
            "--lib",
            "es5",
            "0.ts",
            "--lib",
            "es2015.core, es2015.symbol.wellknown ",
        ]),
        "/proj",
        None,
    );
    assert_eq!(parsed.compiler_options.module, ModuleKind::CommonJS);
    assert_eq!(parsed.compiler_options.target, ScriptTarget::ES5);

    assert_eq!(
        parsed.compiler_options.lib,
        vec![
            "es2015.core".to_string(),
            "es2015.symbol.wellknown".to_string()
        ]
    );
    assert_eq!(parsed.file_names, vec!["/proj/0.ts"]);
}

#[test]
fn test_parse_command_line_lib_empty_followed_by_option() {
    let parsed = parse_command_line(&args(&["0.ts", "--lib", "--sourceMap"]), "/proj", None);
    assert!(parsed.compiler_options.lib.is_empty());
    assert!(parsed.compiler_options.source_map.is_true());
    assert_eq!(parsed.file_names, vec!["/proj/0.ts"]);
}

#[test]
fn test_parse_command_line_unknown_option_error() {
    let parsed = parse_command_line(&args(&["--unknownOpt", "0.ts"]), "/proj", None);
    assert!(has_error_containing(&parsed, "Unknown compiler option"));
    assert!(has_error_containing(&parsed, "unknownOpt"));
}

#[test]
fn test_parse_command_line_explicit_boolean_false() {
    let parsed = parse_command_line(
        &args(&["--strictNullChecks", "false", "0.ts"]),
        "/proj",
        None,
    );
    assert!(parsed.compiler_options.strict_null_checks.is_false());
    assert_eq!(parsed.file_names, vec!["/proj/0.ts"]);
}

#[test]
fn test_parse_command_line_explicit_boolean_true() {
    let parsed = parse_command_line(
        &args(&["--strictNullChecks", "true", "0.ts"]),
        "/proj",
        None,
    );
    assert!(parsed.compiler_options.strict_null_checks.is_true());
}

#[test]
fn test_parse_command_line_implicit_boolean() {
    let parsed = parse_command_line(&args(&["--strictNullChecks"]), "/proj", None);
    assert!(parsed.compiler_options.strict_null_checks.is_true());
}

#[test]
fn test_parse_command_line_non_boolean_after_boolean_flag() {
    let parsed = parse_command_line(&args(&["--noImplicitAny", "t", "0.ts"]), "/proj", None);
    assert!(parsed.compiler_options.no_implicit_any.is_true());
    assert_eq!(parsed.file_names, vec!["/proj/t", "/proj/0.ts"]);
}

#[test]
fn test_parse_command_line_incremental() {
    let parsed = parse_command_line(&args(&["--incremental", "0.ts"]), "/proj", None);
    assert!(parsed.compiler_options.incremental.is_true());
    assert_eq!(parsed.file_names, vec!["/proj/0.ts"]);
}

#[test]
fn test_parse_command_line_ts_build_info_file() {
    let parsed = parse_command_line(
        &args(&["--tsBuildInfoFile", "build.tsbuildinfo", "0.ts"]),
        "/proj",
        None,
    );
    assert_eq!(
        parsed.compiler_options.ts_build_info_file,
        "build.tsbuildinfo"
    );
}

#[test]
fn test_parse_command_line_ts_build_info_file_null() {
    let parsed = parse_command_line(&args(&["--tsBuildInfoFile", "null", "0.ts"]), "/proj", None);
    assert!(parsed.errors.is_empty());
    assert_eq!(parsed.compiler_options.ts_build_info_file, "");
}

#[test]
fn test_parse_command_line_type_roots() {
    let parsed = parse_command_line(
        &args(&["--typeRoots", "t", "bug.ts"]),
        "/home/project",
        None,
    );
    assert_eq!(parsed.compiler_options.type_roots, vec!["t".to_string()]);
    assert_eq!(parsed.file_names, vec!["/home/project/bug.ts"]);
}

#[test]
fn test_parse_command_line_files_in_middle() {
    let parsed = parse_command_line(
        &args(&[
            "--module",
            "commonjs",
            "--target",
            "es5",
            "0.ts",
            "--lib",
            "es5,es2015.symbol.wellknown",
        ]),
        "/proj",
        None,
    );
    assert_eq!(parsed.compiler_options.module, ModuleKind::CommonJS);
    assert_eq!(parsed.compiler_options.target, ScriptTarget::ES5);
    assert_eq!(
        parsed.compiler_options.lib,
        vec!["es5".to_string(), "es2015.symbol.wellknown".to_string()]
    );
    assert_eq!(parsed.file_names, vec!["/proj/0.ts"]);
}

#[test]
fn test_parse_command_line_module_resolution_and_jsx() {
    let parsed = parse_command_line(
        &args(&["--moduleResolution", "node", "--jsx", "react", "0.ts"]),
        "/proj",
        None,
    );
    assert_eq!(
        parsed.compiler_options.module_resolution,
        ModuleResolutionKind::Node10
    );
    assert_eq!(parsed.compiler_options.jsx, JsxEmit::React);
}

#[test]
fn test_response_file_does_not_panic() {
    let parsed = parse_command_line(&args(&["@"]), "/proj", None);
    assert!(!parsed.errors.is_empty());
    assert!(has_error_containing(&parsed, "Cannot read file"));

    let parsed = parse_command_line(&args(&["@blah"]), "/proj", None);
    assert!(!parsed.errors.is_empty());
    assert!(has_error_containing(&parsed, "Cannot read file"));
    assert!(has_error_containing(&parsed, "blah"));
}

#[test]
fn test_response_file_missing_with_fs() {
    let fs = InMemoryFS::new();
    fs.insert_dir("/proj");
    let parsed = parse_command_line(&args(&["@missing.rsp"]), "/proj", Some(&fs));
    assert!(!parsed.errors.is_empty());
    assert!(has_error_containing(&parsed, "Cannot read file"));
}

#[test]
fn test_response_file_propagates_file_names() {
    let fs = InMemoryFS::new();
    fs.insert_dir("/proj");
    fs.insert_file("/proj/args.rsp", "--strict\n0.ts");
    let parsed = parse_command_line(&args(&["@args.rsp"]), "/proj", Some(&fs));
    assert_eq!(parsed.file_names, vec!["/proj/0.ts"]);

    assert!(!has_error_containing(&parsed, "Cannot read file"));
}

#[test]
fn test_response_file_unterminated_quoted_string() {
    let fs = InMemoryFS::new();
    fs.insert_dir("/proj");
    fs.insert_file("/proj/args.rsp", "--outDir \"unterminated path");
    let parsed = parse_command_line(&args(&["@args.rsp"]), "/proj", Some(&fs));

    let has_ts6045 = parsed.errors.iter().any(|e| e.code == 6045);
    assert!(
        has_ts6045,
        "expected TS6045 for unterminated quoted string, got errors: {:?}",
        parsed
            .errors
            .iter()
            .map(|e| (e.code, e.message_args.clone()))
            .collect::<Vec<_>>()
    );

    assert_eq!(
        parsed.compiler_options.out_dir, "unterminated path",
        "unterminated token should still be captured as the option value"
    );
}

#[test]
fn test_strip_jsonc_whitespace_and_empty_object() {
    let stripped = strip_jsonc("   ");
    assert_eq!(stripped.trim(), "");

    let stripped = strip_jsonc("// Comment");
    assert_eq!(stripped.trim(), "");

    let stripped = strip_jsonc("/* Comment */");
    assert_eq!(stripped.trim(), "");

    let stripped = strip_jsonc("{}");
    let v: crate::json::Value = crate::json::from_str(&stripped).unwrap();
    assert!(v.as_object().is_some());
}

#[test]
fn test_strip_jsonc_comments_in_object() {
    let input = r#"{ // Excluded files
            "exclude": [
                // Exclude d.ts
                "file.d.ts"
            ]
        }"#;
    let stripped = strip_jsonc(input);
    let v: crate::json::Value = crate::json::from_str(&stripped).unwrap();
    assert_eq!(v["exclude"][0].as_str(), Some("file.d.ts"));

    let input = r#"{
            /* Excluded
                    Files
            */
            "exclude": [
                /* multiline comments can be in the middle of a line */"file.d.ts"
            ]
        }"#;
    let stripped = strip_jsonc(input);
    let v: crate::json::Value = crate::json::from_str(&stripped).unwrap();
    assert_eq!(v["exclude"][0].as_str(), Some("file.d.ts"));
}

#[test]
fn test_strip_jsonc_keeps_string_content() {
    let input = r#"{
            "exclude": [
                "xx//file.d.ts"
            ]
        }"#;
    let stripped = strip_jsonc(input);
    let v: crate::json::Value = crate::json::from_str(&stripped).unwrap();
    assert_eq!(v["exclude"][0].as_str(), Some("xx//file.d.ts"));

    let input = r#"{
            "exclude": [
                "xx/*file.d.ts*/"
            ]
        }"#;
    let stripped = strip_jsonc(input);
    let v: crate::json::Value = crate::json::from_str(&stripped).unwrap();
    assert_eq!(v["exclude"][0].as_str(), Some("xx/*file.d.ts*/"));
}

#[test]
fn test_strip_jsonc_trailing_comma() {
    let input = r#"{
            "compilerOptions": {
                "target": "ES5",
                "strict": true,
            }
        }"#;
    let stripped = strip_jsonc(input);
    let v: crate::json::Value = crate::json::from_str(&stripped).unwrap();
    assert_eq!(v["compilerOptions"]["target"].as_str(), Some("ES5"));
    assert_eq!(v["compilerOptions"]["strict"].as_bool(), Some(true));
}

#[test]
fn test_parse_tsconfig_extends_merges_options() {
    let fs = InMemoryFS::new();
    fs.insert_dir("/proj");
    fs.insert_file(
        "/proj/base.json",
        r#"{
            "compilerOptions": { "target": "ES2020", "strict": true }
        }"#,
    );
    fs.insert_file(
        "/proj/tsconfig.json",
        r#"{
            "extends": "./base.json",
            "compilerOptions": { "outDir": "./dist" }
        }"#,
    );
    let parsed = get_parsed_command_line_of_config_file(
        "/proj/tsconfig.json",
        &CompilerOptions::default(),
        "/proj",
        &fs,
    );

    assert_eq!(parsed.compiler_options.target, ScriptTarget::ES2020);
    assert!(parsed.compiler_options.strict.is_true());

    assert_eq!(parsed.compiler_options.out_dir, "/proj/dist");

    assert!(parsed.compiler_options.strict_null_checks.is_true());
}

#[test]
fn test_parse_tsconfig_extends_with_own_files_include() {
    let fs = InMemoryFS::new();
    fs.insert_dir("/proj");
    fs.insert_dir("/proj/src");
    fs.insert_file(
        "/proj/base.json",
        r#"{
            "compilerOptions": { "target": "ES2020" }
        }"#,
    );
    fs.insert_file(
        "/proj/tsconfig.json",
        r#"{
            "extends": "./base.json",
            "compilerOptions": { "outDir": "./dist" },
            "include": ["src/**/*"]
        }"#,
    );
    fs.insert_file("/proj/src/a.ts", "export const a = 1;");
    fs.insert_file("/proj/src/b.ts", "export const b = 2;");
    let parsed = get_parsed_command_line_of_config_file(
        "/proj/tsconfig.json",
        &CompilerOptions::default(),
        "/proj",
        &fs,
    );
    assert_eq!(parsed.compiler_options.target, ScriptTarget::ES2020);
    assert_eq!(parsed.compiler_options.out_dir, "/proj/dist");
    assert!(parsed.file_names.contains(&"/proj/src/a.ts".to_string()));
    assert!(parsed.file_names.contains(&"/proj/src/b.ts".to_string()));
}

#[test]
fn test_parse_tsconfig_extends_circular_is_detected() {
    let fs = InMemoryFS::new();
    fs.insert_dir("/proj");
    fs.insert_file(
        "/proj/a.tsconfig.json",
        r#"{ "extends": "./b.tsconfig.json", "compilerOptions": { "target": "ES2020" } }"#,
    );
    fs.insert_file(
        "/proj/b.tsconfig.json",
        r#"{ "extends": "./a.tsconfig.json", "compilerOptions": { "strict": true } }"#,
    );
    let parsed = get_parsed_command_line_of_config_file(
        "/proj/a.tsconfig.json",
        &CompilerOptions::default(),
        "/proj",
        &fs,
    );

    assert!(
        parsed
            .errors
            .iter()
            .any(|e| e.code == CIRCULARITY_DETECTED_WHILE_RESOLVING_CONFIGURATION_COLON_0.code),
        "expected a circularity diagnostic, got errors: {:?}",
        parsed.errors.iter().map(|e| e.code).collect::<Vec<_>>()
    );

    assert_eq!(parsed.compiler_options.target, ScriptTarget::ES2020);
}

#[test]
fn test_parse_tsconfig_extends_as_array_merges_all() {
    let fs = InMemoryFS::new();
    fs.insert_dir("/proj");
    fs.insert_file(
        "/proj/base1.json",
        r#"{ "compilerOptions": { "target": "ES2020", "strict": true } }"#,
    );
    fs.insert_file(
        "/proj/base2.json",
        r#"{ "compilerOptions": { "module": "CommonJS", "declaration": true } }"#,
    );
    fs.insert_file(
        "/proj/tsconfig.json",
        r#"{
            "extends": ["./base1.json", "./base2.json"],
            "compilerOptions": { "outDir": "./dist" },
            "files": []
        }"#,
    );
    let parsed = get_parsed_command_line_of_config_file(
        "/proj/tsconfig.json",
        &CompilerOptions::default(),
        "/proj",
        &fs,
    );
    assert!(
        parsed.errors.is_empty(),
        "unexpected errors: {:?}",
        parsed.errors
    );

    assert_eq!(parsed.compiler_options.target, ScriptTarget::ES2020);
    assert!(parsed.compiler_options.strict.is_true());

    assert_eq!(parsed.compiler_options.module, ModuleKind::CommonJS);
    assert!(parsed.compiler_options.declaration.is_true());

    assert_eq!(parsed.compiler_options.out_dir, "/proj/dist");
}

#[test]
fn test_parse_tsconfig_extends_own_overrides_extended() {
    let fs = InMemoryFS::new();
    fs.insert_dir("/proj");
    fs.insert_file(
        "/proj/base.json",
        r#"{ "compilerOptions": { "strict": true } }"#,
    );
    fs.insert_file(
        "/proj/tsconfig.json",
        r#"{
            "extends": "./base.json",
            "compilerOptions": { "strict": false }
        }"#,
    );
    let parsed = get_parsed_command_line_of_config_file(
        "/proj/tsconfig.json",
        &CompilerOptions::default(),
        "/proj",
        &fs,
    );

    assert!(
        parsed.compiler_options.strict.is_false(),
        "expected own strict=false to override extended strict=true, got {:?}",
        parsed.compiler_options.strict
    );
}

#[test]
fn test_parse_tsconfig_extends_array_last_wins() {
    let fs = InMemoryFS::new();
    fs.insert_dir("/proj");
    fs.insert_file(
        "/proj/base1.json",
        r#"{ "compilerOptions": { "target": "ES2020" } }"#,
    );
    fs.insert_file(
        "/proj/base2.json",
        r#"{ "compilerOptions": { "target": "ES2015" } }"#,
    );
    fs.insert_file(
        "/proj/tsconfig.json",
        r#"{ "extends": ["./base1.json", "./base2.json"] }"#,
    );
    let parsed = get_parsed_command_line_of_config_file(
        "/proj/tsconfig.json",
        &CompilerOptions::default(),
        "/proj",
        &fs,
    );

    assert_eq!(
        parsed.compiler_options.target,
        ScriptTarget::ES2015,
        "expected last extends entry (base2/ES2015) to win, got {:?}",
        parsed.compiler_options.target
    );
}

#[test]
fn test_parse_tsconfig_extends_command_line_overrides_own() {
    let fs = InMemoryFS::new();
    fs.insert_dir("/proj");
    fs.insert_file(
        "/proj/tsconfig.json",
        r#"{ "compilerOptions": { "strict": true } }"#,
    );
    let mut base = CompilerOptions::default();
    base.strict = Tristate::False;
    let parsed = get_parsed_command_line_of_config_file("/proj/tsconfig.json", &base, "/proj", &fs);

    assert!(
        parsed.compiler_options.strict.is_false(),
        "expected command-line strict=false to override config strict=true, got {:?}",
        parsed.compiler_options.strict
    );
}

#[test]
fn test_parse_tsconfig_extends_include_first_extended_wins() {
    let fs = InMemoryFS::new();
    fs.insert_dir("/proj");
    fs.insert_dir("/proj/src1");
    fs.insert_dir("/proj/src2");
    fs.insert_file("/proj/src1/a.ts", "export const a = 1;");
    fs.insert_file("/proj/src2/b.ts", "export const b = 2;");
    fs.insert_file("/proj/base1.json", r#"{ "include": ["src1/**/*"] }"#);
    fs.insert_file("/proj/base2.json", r#"{ "include": ["src2/**/*"] }"#);
    fs.insert_file(
        "/proj/tsconfig.json",
        r#"{ "extends": ["./base1.json", "./base2.json"] }"#,
    );
    let parsed = get_parsed_command_line_of_config_file(
        "/proj/tsconfig.json",
        &CompilerOptions::default(),
        "/proj",
        &fs,
    );

    assert!(
        parsed.file_names.contains(&"/proj/src1/a.ts".to_string()),
        "expected first extended include (src1) to win, got {:?}",
        parsed.file_names
    );
    assert!(
        !parsed.file_names.contains(&"/proj/src2/b.ts".to_string()),
        "expected second extended include (src2) to be suppressed, got {:?}",
        parsed.file_names
    );
}

#[test]
fn test_parse_tsconfig_extends_resolves_json_suffix() {
    let fs = InMemoryFS::new();
    fs.insert_dir("/proj");
    fs.insert_file(
        "/proj/base.json",
        r#"{ "compilerOptions": { "strict": true } }"#,
    );
    fs.insert_file("/proj/tsconfig.json", r#"{ "extends": "./base" }"#);
    let parsed = get_parsed_command_line_of_config_file(
        "/proj/tsconfig.json",
        &CompilerOptions::default(),
        "/proj",
        &fs,
    );

    assert!(
        parsed.compiler_options.strict.is_true(),
        "expected extends ./base to resolve to ./base.json and inherit strict=true, got {:?}",
        parsed.compiler_options.strict
    );
}

#[test]
fn test_parse_tsconfig_full_compiler_options() {
    let fs = InMemoryFS::new();
    fs.insert_dir("/apath");
    fs.insert_dir("/apath/src");
    fs.insert_dir("/apath/node_modules");
    fs.insert_dir("/apath/dist");
    fs.insert_file(
        "/apath/tsconfig.json",
        r#"{
            "compilerOptions": {
                "outDir": "./dist",
                "strict": true,
                "noImplicitAny": true,
                "target": "ES2017",
                "module": "ESNext",
                "moduleResolution": "bundler",
                "moduleDetection": "auto",
                "jsx": "react"
            },
            "files": ["/apath/src/index.ts", "/apath/src/app.ts"],
            "include": ["/apath/src/**/*"],
            "exclude": ["/apath/node_modules", "/apath/dist"]
        }"#,
    );
    fs.insert_file("/apath/src/index.ts", "");
    fs.insert_file("/apath/src/app.ts", "");
    fs.insert_file("/apath/node_modules/module.ts", "");
    fs.insert_file("/apath/dist/output.js", "");
    let parsed = get_parsed_command_line_of_config_file(
        "/apath/tsconfig.json",
        &CompilerOptions::default(),
        "/apath",
        &fs,
    );
    assert_eq!(parsed.compiler_options.target, ScriptTarget::ES2017);
    assert_eq!(parsed.compiler_options.module, ModuleKind::ESNext);
    assert_eq!(
        parsed.compiler_options.module_resolution,
        ModuleResolutionKind::Bundler
    );
    assert_eq!(parsed.compiler_options.jsx, JsxEmit::React);
    assert!(parsed.compiler_options.strict.is_true());
    assert!(parsed.compiler_options.no_implicit_any.is_true());
    assert_eq!(parsed.compiler_options.out_dir, "/apath/dist");

    assert!(
        parsed
            .file_names
            .contains(&"/apath/src/index.ts".to_string())
    );
    assert!(parsed.file_names.contains(&"/apath/src/app.ts".to_string()));

    assert!(!parsed.file_names.iter().any(|f| f.contains("node_modules")));
}

#[test]
fn test_parse_tsconfig_null_enum_options() {
    let fs = InMemoryFS::new();
    fs.insert_dir("/proj");
    fs.insert_file(
        "/proj/tsconfig.json",
        r#"{
            "compilerOptions": {
                "target": null,
                "module": null
            }
        }"#,
    );
    fs.insert_file("/proj/app.ts", "");
    let parsed = get_parsed_command_line_of_config_file(
        "/proj/tsconfig.json",
        &CompilerOptions::default(),
        "/proj",
        &fs,
    );
    assert!(parsed.errors.is_empty());
}

#[test]
fn test_parse_tsconfig_empty_types_array() {
    let fs = InMemoryFS::new();
    fs.insert_dir("/proj");
    fs.insert_file(
        "/proj/tsconfig.json",
        r#"{
            "compilerOptions": {
                "types": []
            }
        }"#,
    );
    fs.insert_file("/proj/app.ts", "");
    let parsed = get_parsed_command_line_of_config_file(
        "/proj/tsconfig.json",
        &CompilerOptions::default(),
        "/proj",
        &fs,
    );
    assert!(parsed.compiler_options.types.is_empty());
}

#[test]
fn test_parse_tsconfig_include_with_exclude() {
    let fs = InMemoryFS::new();
    fs.insert_dir("/proj");
    fs.insert_dir("/proj/src");
    fs.insert_dir("/proj/src/tests");
    fs.insert_file(
        "/proj/tsconfig.json",
        r#"{
            "include": ["src/**/*.ts"],
            "exclude": ["**/tests/**"]
        }"#,
    );
    fs.insert_file("/proj/src/a.ts", "");
    fs.insert_file("/proj/src/b.ts", "");
    fs.insert_file("/proj/src/tests/skip.ts", "");
    let parsed = get_parsed_command_line_of_config_file(
        "/proj/tsconfig.json",
        &CompilerOptions::default(),
        "/proj",
        &fs,
    );
    assert!(parsed.file_names.contains(&"/proj/src/a.ts".to_string()));
    assert!(parsed.file_names.contains(&"/proj/src/b.ts".to_string()));

    assert!(
        !parsed
            .file_names
            .contains(&"/proj/src/tests/skip.ts".to_string())
    );
}

#[test]
fn test_parse_tsconfig_literal_directory_include_recurses() {
    let fs = InMemoryFS::new();
    fs.insert_dir("/proj");
    fs.insert_dir("/proj/src");
    fs.insert_dir("/proj/src/nested");
    fs.insert_file(
        "/proj/tsconfig.json",
        r#"{
                "include": ["src"]
            }"#,
    );
    fs.insert_file("/proj/src/a.ts", "");
    fs.insert_file("/proj/src/nested/b.tsx", "");
    fs.insert_file("/proj/src/nested/ignore.txt", "");
    let parsed = get_parsed_command_line_of_config_file(
        "/proj/tsconfig.json",
        &CompilerOptions::default(),
        "/proj",
        &fs,
    );
    assert!(parsed.file_names.contains(&"/proj/src/a.ts".to_string()));
    assert!(
        parsed
            .file_names
            .contains(&"/proj/src/nested/b.tsx".to_string())
    );
    assert!(!parsed.file_names.iter().any(|f| f.ends_with("ignore.txt")));
}

#[test]
fn test_parse_tsconfig_skips_node_modules_directory() {
    let fs = InMemoryFS::new();
    fs.insert_dir("/proj");
    fs.insert_dir("/proj/node_modules");
    fs.insert_dir("/proj/folder");
    fs.insert_file("/proj/tsconfig.json", "{}");
    fs.insert_file("/proj/node_modules/a.ts", "");
    fs.insert_file("/proj/d.ts", "");
    fs.insert_file("/proj/folder/e.ts", "");
    let parsed = get_parsed_command_line_of_config_file(
        "/proj/tsconfig.json",
        &CompilerOptions::default(),
        "/proj",
        &fs,
    );
    assert!(!parsed.file_names.iter().any(|f| f.contains("node_modules")));
    assert!(parsed.file_names.contains(&"/proj/d.ts".to_string()));
    assert!(parsed.file_names.contains(&"/proj/folder/e.ts".to_string()));
}

#[test]
fn test_parse_tsconfig_files_empty_does_not_default_include() {
    let fs = InMemoryFS::new();
    fs.insert_dir("/proj");
    fs.insert_dir("/proj/src");
    fs.insert_file(
        "/proj/tsconfig.json",
        r#"{
                "files": [],
                "references": [{ "path": "./tsconfig.app.json" }]
            }"#,
    );
    fs.insert_file("/proj/src/a.ts", "");
    let parsed = get_parsed_command_line_of_config_file(
        "/proj/tsconfig.json",
        &CompilerOptions::default(),
        "/proj",
        &fs,
    );
    assert!(parsed.has_files_spec);
    assert!(parsed.file_names.is_empty());
}

#[test]
fn test_tsconfig_no_inputs_emits_ts18003() {
    let fs = InMemoryFS::new();
    fs.insert_dir("/proj");
    fs.insert_file(
        "/proj/tsconfig.json",
        r#"{ "compilerOptions": { "outDir": "./dist" } }"#,
    );
    let parsed = get_parsed_command_line_of_config_file(
        "/proj/tsconfig.json",
        &CompilerOptions::default(),
        "/proj",
        &fs,
    );
    assert!(parsed.file_names.is_empty());
    assert!(
        parsed.errors.iter().any(|d| d.code == 18003),
        "expected TS18003, got errors: {:?}",
        parsed.errors
    );
}

#[test]
fn test_tsconfig_no_inputs_suppressed_by_files_key() {
    let fs = InMemoryFS::new();
    fs.insert_dir("/proj");
    fs.insert_file("/proj/src/a.ts", "");
    fs.insert_file("/proj/tsconfig.json", r#"{ "files": [] }"#);
    let parsed = get_parsed_command_line_of_config_file(
        "/proj/tsconfig.json",
        &CompilerOptions::default(),
        "/proj",
        &fs,
    );
    assert!(parsed.file_names.is_empty());
    assert!(
        !parsed.errors.iter().any(|d| d.code == 18003),
        "did not expect TS18003, got errors: {:?}",
        parsed.errors
    );
}

#[test]
fn test_tsconfig_no_inputs_suppressed_by_references_key() {
    let fs = InMemoryFS::new();
    fs.insert_dir("/proj");
    fs.insert_file(
        "/proj/tsconfig.json",
        r#"{ "references": [{ "path": "./other.json" }] }"#,
    );
    let parsed = get_parsed_command_line_of_config_file(
        "/proj/tsconfig.json",
        &CompilerOptions::default(),
        "/proj",
        &fs,
    );
    assert!(parsed.file_names.is_empty());
    assert!(
        !parsed.errors.iter().any(|d| d.code == 18003),
        "did not expect TS18003, got errors: {:?}",
        parsed.errors
    );
}

#[test]
fn test_tsconfig_references_parsed_as_typed_project_reference() {
    let fs = InMemoryFS::new();
    fs.insert_dir("/proj");
    fs.insert_dir("/proj/test");
    fs.insert_file(
        "/proj/tsconfig.json",
        r#"{ "references": [{ "path": "./test" }, { "path": "./other.json" }] }"#,
    );
    let parsed = get_parsed_command_line_of_config_file(
        "/proj/tsconfig.json",
        &CompilerOptions::default(),
        "/proj",
        &fs,
    );
    assert_eq!(parsed.references.len(), 2);
    assert_eq!(parsed.references[0].original_path, "./test");
    assert_eq!(parsed.references[0].path, "/proj/test");
    assert!(!parsed.references[0].circular);
    assert_eq!(parsed.references[1].original_path, "./other.json");
    assert_eq!(parsed.references[1].path, "/proj/other.json");
}

#[test]
fn test_parse_tsconfig_excludes_out_dir_by_default() {
    let fs = InMemoryFS::new();
    fs.insert_dir("/proj");
    fs.insert_dir("/proj/src");
    fs.insert_dir("/proj/dist");
    fs.insert_file(
        "/proj/tsconfig.json",
        r#"{
                "compilerOptions": { "outDir": "dist" }
            }"#,
    );
    fs.insert_file("/proj/src/a.ts", "");
    fs.insert_file("/proj/dist/a.d.ts", "");
    let parsed = get_parsed_command_line_of_config_file(
        "/proj/tsconfig.json",
        &CompilerOptions::default(),
        "/proj",
        &fs,
    );
    assert!(parsed.file_names.contains(&"/proj/src/a.ts".to_string()));
    assert!(!parsed.file_names.iter().any(|f| f.contains("/dist/")));
}

#[test]
fn test_parse_tsconfig_explicit_exclude_overrides_out_dir_default() {
    let fs = InMemoryFS::new();
    fs.insert_dir("/proj");
    fs.insert_dir("/proj/dist");
    fs.insert_file(
        "/proj/tsconfig.json",
        r#"{
                "compilerOptions": { "outDir": "dist" },
                "exclude": ["obj"]
            }"#,
    );
    fs.insert_file("/proj/dist/a.d.ts", "");
    let parsed = get_parsed_command_line_of_config_file(
        "/proj/tsconfig.json",
        &CompilerOptions::default(),
        "/proj",
        &fs,
    );
    assert!(parsed.has_exclude_spec);
    assert!(parsed.file_names.contains(&"/proj/dist/a.d.ts".to_string()));
}

#[test]
fn test_parse_tsconfig_skips_common_package_directories() {
    let fs = InMemoryFS::new();
    fs.insert_dir("/proj");
    fs.insert_dir("/proj/node_modules");
    fs.insert_dir("/proj/bower_components");
    fs.insert_dir("/proj/jspm_packages");
    fs.insert_file("/proj/tsconfig.json", "{}");
    fs.insert_file("/proj/node_modules/a.ts", "");
    fs.insert_file("/proj/bower_components/b.ts", "");
    fs.insert_file("/proj/jspm_packages/c.ts", "");
    fs.insert_file("/proj/d.ts", "");
    let parsed = get_parsed_command_line_of_config_file(
        "/proj/tsconfig.json",
        &CompilerOptions::default(),
        "/proj",
        &fs,
    );
    assert!(!parsed.file_names.iter().any(|f| f.contains("node_modules")));
    assert!(
        !parsed
            .file_names
            .iter()
            .any(|f| f.contains("bower_components"))
    );
    assert!(
        !parsed
            .file_names
            .iter()
            .any(|f| f.contains("jspm_packages"))
    );
    assert!(parsed.file_names.contains(&"/proj/d.ts".to_string()));
}

#[test]
fn test_parse_tsconfig_skips_git_directory() {
    let fs = InMemoryFS::new();
    fs.insert_dir("/proj");
    fs.insert_dir("/proj/.git");
    fs.insert_file("/proj/tsconfig.json", "{}");
    fs.insert_file("/proj/.git/a.ts", "");
    fs.insert_file("/proj/test.ts", "");
    let parsed = get_parsed_command_line_of_config_file(
        "/proj/tsconfig.json",
        &CompilerOptions::default(),
        "/proj",
        &fs,
    );
    assert!(!parsed.file_names.iter().any(|f| f.contains(".git")));
    assert!(parsed.file_names.contains(&"/proj/test.ts".to_string()));
}

#[test]
fn test_parse_tsconfig_missing_config_file_error() {
    let fs = InMemoryFS::new();
    fs.insert_dir("/proj");
    let parsed = get_parsed_command_line_of_config_file(
        "/proj/missing.json",
        &CompilerOptions::default(),
        "/proj",
        &fs,
    );
    assert!(!parsed.errors.is_empty());
    assert!(has_error_containing(&parsed, "Cannot find"));
}

#[test]
fn test_parse_tsconfig_invalid_json_error() {
    let fs = InMemoryFS::new();
    fs.insert_dir("/proj");
    fs.insert_file("/proj/tsconfig.json", "{ this is not json");
    let parsed = get_parsed_command_line_of_config_file(
        "/proj/tsconfig.json",
        &CompilerOptions::default(),
        "/proj",
        &fs,
    );
    assert!(!parsed.errors.is_empty());
    assert!(has_error_containing(&parsed, "Failed to parse"));
}

#[test]
fn test_parse_tsconfig_command_line_overrides_config() {
    let fs = InMemoryFS::new();
    fs.insert_dir("/proj");
    fs.insert_file(
        "/proj/tsconfig.json",
        r#"{
            "compilerOptions": { "target": "ES2017", "strict": true }
        }"#,
    );
    fs.insert_file("/proj/app.ts", "");
    let mut base = CompilerOptions::default();
    base.target = ScriptTarget::ES2022;
    let parsed = get_parsed_command_line_of_config_file("/proj/tsconfig.json", &base, "/proj", &fs);

    assert_eq!(parsed.compiler_options.target, ScriptTarget::ES2022);
    assert!(parsed.compiler_options.strict.is_true());
}

#[test]
fn test_parsed_command_line_literal_file_list_dedup() {
    let fs = InMemoryFS::new();
    fs.insert_dir("/dev");
    fs.insert_file("/dev/a.ts", "");
    fs.insert_file("/dev/b.ts", "");
    fs.insert_file(
        "/dev/tsconfig.json",
        r#"{
            "files": ["a.ts", "a.ts", "b.ts"]
        }"#,
    );
    let parsed = get_parsed_command_line_of_config_file(
        "/dev/tsconfig.json",
        &CompilerOptions::default(),
        "/dev",
        &fs,
    );

    assert_eq!(
        parsed.file_names,
        vec!["/dev/a.ts".to_string(), "/dev/b.ts".to_string()]
    );
}

#[test]
fn test_parsed_command_line_files_not_removed_by_exclude() {
    let fs = InMemoryFS::new();
    fs.insert_dir("/dev");
    fs.insert_file("/dev/a.ts", "");
    fs.insert_file("/dev/b.ts", "");
    fs.insert_file(
        "/dev/tsconfig.json",
        r#"{
            "files": ["a.ts", "b.ts"],
            "exclude": ["b.ts"]
        }"#,
    );
    let parsed = get_parsed_command_line_of_config_file(
        "/dev/tsconfig.json",
        &CompilerOptions::default(),
        "/dev",
        &fs,
    );
    assert!(parsed.file_names.contains(&"/dev/a.ts".to_string()));
    assert!(parsed.file_names.contains(&"/dev/b.ts".to_string()));
}

#[test]
fn test_parsed_command_line_literal_include_matches_files() {
    let fs = InMemoryFS::new();
    fs.insert_dir("/dev");
    fs.insert_file("/dev/a.ts", "");
    fs.insert_file("/dev/b.ts", "");
    fs.insert_file(
        "/dev/tsconfig.json",
        r#"{
            "include": ["a.ts", "b.ts"]
        }"#,
    );
    let parsed = get_parsed_command_line_of_config_file(
        "/dev/tsconfig.json",
        &CompilerOptions::default(),
        "/dev",
        &fs,
    );
    assert!(parsed.file_names.contains(&"/dev/a.ts".to_string()));
    assert!(parsed.file_names.contains(&"/dev/b.ts".to_string()));
}

#[test]
fn test_wildcard_include_dot_prefixed_with_dot_dir_exclude() {
    let fs = InMemoryFS::new();
    fs.insert_dir("/home/projects/monorepo/apps/web");
    fs.insert_dir("/home/projects/monorepo/apps/web/app");
    fs.insert_file(
        "/home/projects/monorepo/apps/web/tsconfig.json",
        r#"{
                "include": ["app/**/*.ts", "app/**/*.tsx"],
                "exclude": ["**/node_modules", "**/.*/", "build"]
            }"#,
    );
    fs.insert_file("/home/projects/monorepo/apps/web/app/a.ts", "");
    fs.insert_file("/home/projects/monorepo/apps/web/app/b.tsx", "");
    let parsed = get_parsed_command_line_of_config_file(
        "/home/projects/monorepo/apps/web/tsconfig.json",
        &CompilerOptions::default(),
        "/home/projects/monorepo/apps/web",
        &fs,
    );
    assert!(
        parsed
            .file_names
            .contains(&"/home/projects/monorepo/apps/web/app/a.ts".to_string())
    );
    assert!(
        parsed
            .file_names
            .contains(&"/home/projects/monorepo/apps/web/app/b.tsx".to_string())
    );
}

#[test]
fn test_wildcard_include_non_ascii_paths() {
    let fs = InMemoryFS::new();
    fs.insert_dir("/Users/ユーザー/プロジェクト");
    fs.insert_dir("/Users/ユーザー/プロジェクト/src");
    fs.insert_file(
        "/Users/ユーザー/プロジェクト/tsconfig.json",
        r#"{
                "include": ["src/**/*.ts"],
                "exclude": ["テスト"]
            }"#,
    );
    fs.insert_file("/Users/ユーザー/プロジェクト/src/a.ts", "");
    let parsed = get_parsed_command_line_of_config_file(
        "/Users/ユーザー/プロジェクト/tsconfig.json",
        &CompilerOptions::default(),
        "/Users/ユーザー/プロジェクト",
        &fs,
    );
    assert!(parsed.file_names.iter().any(|f| f.ends_with("/src/a.ts")));
}

#[test]
fn test_options_declarations_non_empty_and_named() {
    assert!(!OPTIONS.is_empty());
    for o in OPTIONS.iter() {
        assert!(!o.name.is_empty(), "found an option with an empty name");
    }

    let names: std::collections::HashSet<&str> = OPTIONS.iter().map(|o| o.name).collect();
    for required in [
        "help",
        "version",
        "build",
        "watch",
        "target",
        "module",
        "jsx",
        "lib",
        "strict",
        "noEmit",
        "project",
        "tsBuildInfoFile",
        "incremental",
        "moduleResolution",
        "typeRoots",
    ] {
        assert!(
            names.contains(required),
            "missing option declaration: {required}"
        );
    }
}

#[test]
fn test_option_decls_short_names_unique_or_known() {
    assert_eq!(find_option("h").map(|o| o.name), Some("help"));
    assert_eq!(find_option("v").map(|o| o.name), Some("version"));
    assert_eq!(find_option("b").map(|o| o.name), Some("build"));
    assert_eq!(find_option("w").map(|o| o.name), Some("watch"));
    assert_eq!(find_option("p").map(|o| o.name), Some("project"));
    assert_eq!(find_option("t").map(|o| o.name), Some("target"));
    assert_eq!(find_option("m").map(|o| o.name), Some("module"));
    assert_eq!(find_option("d").map(|o| o.name), Some("declaration"));
}

fn diag_contains(errors: &[Diagnostic], needle: &str) -> bool {
    errors.iter().any(|e| {
        e.message_args.iter().any(|a| a.contains(needle))
            || e.message.map(|m| m.text.contains(needle)).unwrap_or(false)
    })
}

#[test]
fn test_case_insensitive_option_lookup_cli() {
    let parsed = parse_command_line(&args(&["--Target", "ES2020", "0.ts"]), "/proj", None);
    assert_eq!(parsed.compiler_options.target, ScriptTarget::ES2020);
    assert!(!has_error_containing(&parsed, "Unknown compiler option"));

    let parsed = parse_command_line(
        &args(&["--Module", "commonjs", "--Jsx", "react", "0.ts"]),
        "/proj",
        None,
    );
    assert_eq!(parsed.compiler_options.module, ModuleKind::CommonJS);
    assert_eq!(parsed.compiler_options.jsx, JsxEmit::React);
}

#[test]
fn test_case_insensitive_short_name_lookup() {
    let parsed = parse_command_line(&args(&["-P", "tsconfig.json"]), "/proj", None);
    assert_eq!(parsed.compiler_options.project, "tsconfig.json");
}

#[test]
fn test_alternate_mode_build_option_in_compiler_mode() {
    let parsed = parse_command_line(&args(&["--dry", "0.ts"]), "/proj", None);
    assert!(diag_contains(
        &parsed.errors,
        "may only be used with '--build'"
    ));
    assert!(!diag_contains(&parsed.errors, "Unknown compiler option"));
}

#[test]
fn test_alternate_mode_verbose_in_compiler_mode() {
    let parsed = parse_command_line(&args(&["--verbose"]), "/proj", None);
    assert!(diag_contains(
        &parsed.errors,
        "may only be used with '--build'"
    ));
}

#[test]
fn test_tsconfig_only_option_on_cli_emits_diagnostic() {
    let parsed = parse_command_line(&args(&["--composite", "0.ts"]), "/proj", None);
    assert!(has_error_containing(&parsed, "tsconfig.json"));
    assert!(has_error_containing(&parsed, "composite"));
    assert!(!parsed.compiler_options.composite.is_true());
}

#[test]
fn test_tsconfig_only_boolean_accepts_false() {
    let parsed = parse_command_line(&args(&["--composite", "false", "0.ts"]), "/proj", None);
    assert!(!has_error_containing(&parsed, "tsconfig.json"));
    assert!(parsed.compiler_options.composite.is_false());
}

#[test]
fn test_tsconfig_only_boolean_accepts_null() {
    let parsed = parse_command_line(&args(&["--composite", "null", "0.ts"]), "/proj", None);
    assert!(!has_error_containing(&parsed, "tsconfig.json"));
}

#[test]
fn test_invalid_enum_value_target() {
    let parsed = parse_command_line(&args(&["--target", "es99", "0.ts"]), "/proj", None);
    assert!(has_error_containing(&parsed, "Argument for"));
    assert!(has_error_containing(&parsed, "--target"));
    assert!(has_error_containing(&parsed, "es5"));
    assert_eq!(parsed.compiler_options.target, ScriptTarget::None);
}

#[test]
fn test_invalid_enum_value_module() {
    let parsed = parse_command_line(&args(&["--module", "nonsense", "0.ts"]), "/proj", None);
    assert!(has_error_containing(&parsed, "Argument for"));
    assert!(has_error_containing(&parsed, "commonjs"));
    assert_eq!(parsed.compiler_options.module, ModuleKind::None);
}

#[test]
fn test_valid_enum_value_case_insensitive() {
    let parsed = parse_command_line(&args(&["--target", "ES2020", "0.ts"]), "/proj", None);
    assert!(!has_error_containing(&parsed, "Argument for"));
    assert_eq!(parsed.compiler_options.target, ScriptTarget::ES2020);
}

#[test]
fn test_min_value_violation_builders() {
    let parsed = parse_build_command_line(&args(&["--build", "--builders", "0"]), "/proj", None);
    assert!(diag_contains(
        &parsed.errors,
        "requires value to be greater"
    ));
    assert!(diag_contains(&parsed.errors, "builders"));
    assert!(diag_contains(&parsed.errors, "1"));
}

#[test]
fn test_min_value_accepted_builders() {
    let parsed = parse_build_command_line(&args(&["--build", "--builders", "2"]), "/proj", None);
    assert!(!diag_contains(
        &parsed.errors,
        "requires value to be greater"
    ));
    assert_eq!(parsed.build_options.builders, Some(2));
}

#[test]
fn test_case_mismatch_in_tsconfig_json_emits_did_you_mean() {
    let fs = InMemoryFS::new();
    fs.insert_dir("/proj");
    fs.insert_dir("/proj/src");
    fs.insert_file(
        "/proj/tsconfig.json",
        r#"{
            "compilerOptions": { "Target": "es2020", "noEmit": true },
            "files": ["src/a.ts"]
        }"#,
    );
    fs.insert_file("/proj/src/a.ts", "export const x = 1;");
    let parsed = get_parsed_command_line_of_config_file(
        "/proj/tsconfig.json",
        &CompilerOptions::default(),
        "/proj",
        &fs,
    );
    assert!(has_error_containing(&parsed, "Did you mean"));
    assert!(has_error_containing(&parsed, "Target"));
    assert!(has_error_containing(&parsed, "target"));

    assert_eq!(parsed.compiler_options.target, ScriptTarget::None);

    assert!(parsed.compiler_options.no_emit.is_true());
}

#[test]
fn test_tsconfig_json_correct_case_no_did_you_mean() {
    let fs = InMemoryFS::new();
    fs.insert_dir("/proj");
    fs.insert_dir("/proj/src");
    fs.insert_file(
        "/proj/tsconfig.json",
        r#"{
            "compilerOptions": { "target": "es2020", "noEmit": true },
            "files": ["src/a.ts"]
        }"#,
    );
    fs.insert_file("/proj/src/a.ts", "export const x = 1;");
    let parsed = get_parsed_command_line_of_config_file(
        "/proj/tsconfig.json",
        &CompilerOptions::default(),
        "/proj",
        &fs,
    );
    assert!(!has_error_containing(&parsed, "Did you mean"));
    assert_eq!(parsed.compiler_options.target, ScriptTarget::ES2020);
}

#[test]
fn test_enum_values_declared_on_all_enum_options() {
    for o in OPTIONS.iter().chain(BUILD_OPTIONS.iter()) {
        if o.kind == OptionKind::Enum {
            assert!(
                o.enum_values.is_some(),
                "enum option '{}' must declare enum_values",
                o.name
            );
        }
    }
}

#[test]
fn test_tsconfig_only_and_min_value_flags_set() {
    let composite = find_option("composite").expect("composite must exist");
    assert!(composite.is_tsconfig_only);
    let paths = find_option("paths").expect("paths must exist");
    assert!(paths.is_tsconfig_only);
    let builders = find_build_only_option("builders").expect("builders must exist");
    assert_eq!(builders.min_value, Some(1));
}

#[test]
fn test_config_dir_substitution_out_dir() {
    let fs = InMemoryFS::new();
    fs.insert_dir("/proj");
    fs.insert_file(
        "/proj/tsconfig.json",
        r#"{ "compilerOptions": { "outDir": "${configDir}/out" } }"#,
    );
    let parsed = get_parsed_command_line_of_config_file(
        "/proj/tsconfig.json",
        &CompilerOptions::default(),
        "/proj",
        &fs,
    );
    assert_eq!(
        parsed.compiler_options.out_dir, "/proj/out",
        "expected ${{configDir}}/out to resolve to /proj/out, got {}",
        parsed.compiler_options.out_dir
    );
}

#[test]
fn test_config_dir_substitution_root_dir() {
    let fs = InMemoryFS::new();
    fs.insert_dir("/proj");
    fs.insert_file(
        "/proj/tsconfig.json",
        r#"{ "compilerOptions": { "rootDir": "${configDir}/src" } }"#,
    );
    let parsed = get_parsed_command_line_of_config_file(
        "/proj/tsconfig.json",
        &CompilerOptions::default(),
        "/proj",
        &fs,
    );
    assert_eq!(parsed.compiler_options.root_dir, "/proj/src");
}

#[test]
fn test_config_dir_substitution_case_insensitive_detection() {
    let fs = InMemoryFS::new();
    fs.insert_dir("/proj");
    fs.insert_file(
        "/proj/tsconfig.json",
        r#"{ "compilerOptions": { "outDir": "${configDir}/out" } }"#,
    );
    let parsed = get_parsed_command_line_of_config_file(
        "/proj/tsconfig.json",
        &CompilerOptions::default(),
        "/proj",
        &fs,
    );

    assert_eq!(parsed.compiler_options.out_dir, "/proj/out");
}

#[test]
fn test_config_dir_substitution_declaration_dir_and_ts_build_info() {
    let fs = InMemoryFS::new();
    fs.insert_dir("/proj");
    fs.insert_file(
        "/proj/tsconfig.json",
        r#"{ "compilerOptions": {
                "declarationDir": "${configDir}/decls",
                "tsBuildInfoFile": "${configDir}/build.tsbuildinfo"
            } }"#,
    );
    let parsed = get_parsed_command_line_of_config_file(
        "/proj/tsconfig.json",
        &CompilerOptions::default(),
        "/proj",
        &fs,
    );
    assert_eq!(parsed.compiler_options.declaration_dir, "/proj/decls");
    assert_eq!(
        parsed.compiler_options.ts_build_info_file,
        "/proj/build.tsbuildinfo"
    );
}

#[test]
fn test_config_dir_substitution_root_dirs_array() {
    let fs = InMemoryFS::new();
    fs.insert_dir("/proj");
    fs.insert_file(
        "/proj/tsconfig.json",
        r#"{ "compilerOptions": {
                "rootDirs": ["${configDir}/src", "${configDir}/lib"]
            } }"#,
    );
    let parsed = get_parsed_command_line_of_config_file(
        "/proj/tsconfig.json",
        &CompilerOptions::default(),
        "/proj",
        &fs,
    );
    assert_eq!(
        parsed.compiler_options.root_dirs,
        vec!["/proj/src".to_string(), "/proj/lib".to_string()]
    );
}

#[test]
fn test_config_dir_substitution_paths() {
    let fs = InMemoryFS::new();
    fs.insert_dir("/proj");
    fs.insert_file(
        "/proj/tsconfig.json",
        r#"{ "compilerOptions": {
                "baseUrl": ".",
                "paths": {
                    "@/*": ["${configDir}/src/*"],
                    "lib/*": ["${configDir}/lib/*"]
                }
            } }"#,
    );
    let parsed = get_parsed_command_line_of_config_file(
        "/proj/tsconfig.json",
        &CompilerOptions::default(),
        "/proj",
        &fs,
    );
    let paths = parsed.compiler_options.paths.expect("paths should be set");
    assert_eq!(paths.get("@/*").unwrap(), &vec!["/proj/src/*".to_string()]);
    assert_eq!(
        paths.get("lib/*").unwrap(),
        &vec!["/proj/lib/*".to_string()]
    );
}

#[test]
fn test_config_dir_substitution_include() {
    let fs = InMemoryFS::new();
    fs.insert_dir("/proj");
    fs.insert_dir("/proj/src");
    fs.insert_file("/proj/src/index.ts", "");
    fs.insert_file(
        "/proj/tsconfig.json",
        r#"{ "include": ["${configDir}/src"] }"#,
    );
    let parsed = get_parsed_command_line_of_config_file(
        "/proj/tsconfig.json",
        &CompilerOptions::default(),
        "/proj",
        &fs,
    );
    assert!(
        parsed.file_names.iter().any(|f| f == "/proj/src/index.ts"),
        "expected /proj/src/index.ts in file_names, got {:?}",
        parsed.file_names
    );
}

#[test]
fn test_config_dir_substitution_files() {
    let fs = InMemoryFS::new();
    fs.insert_dir("/proj");
    fs.insert_file("/proj/main.ts", "");
    fs.insert_file(
        "/proj/tsconfig.json",
        r#"{ "files": ["${configDir}/main.ts"] }"#,
    );
    let parsed = get_parsed_command_line_of_config_file(
        "/proj/tsconfig.json",
        &CompilerOptions::default(),
        "/proj",
        &fs,
    );
    assert!(
        parsed.file_names.iter().any(|f| f == "/proj/main.ts"),
        "expected /proj/main.ts in file_names, got {:?}",
        parsed.file_names
    );
}

#[test]
fn test_config_dir_substitution_exclude() {
    let fs = InMemoryFS::new();
    fs.insert_dir("/proj");
    fs.insert_dir("/proj/src");
    fs.insert_dir("/proj/dist");
    fs.insert_file("/proj/src/index.ts", "");
    fs.insert_file("/proj/dist/output.js", "");
    fs.insert_file(
        "/proj/tsconfig.json",
        r#"{ "include": ["${configDir}/src/**/*"], "exclude": ["${configDir}/dist"] }"#,
    );
    let parsed = get_parsed_command_line_of_config_file(
        "/proj/tsconfig.json",
        &CompilerOptions::default(),
        "/proj",
        &fs,
    );
    assert!(
        parsed.file_names.iter().any(|f| f == "/proj/src/index.ts"),
        "expected /proj/src/index.ts in file_names, got {:?}",
        parsed.file_names
    );
    assert!(
        !parsed.file_names.iter().any(|f| f.contains("dist")),
        "expected dist/ files to be excluded, got {:?}",
        parsed.file_names
    );
}

#[test]
fn test_config_dir_substitution_with_extends() {
    let fs = InMemoryFS::new();
    fs.insert_dir("/proj");
    fs.insert_dir("/proj/base");
    fs.insert_file(
        "/proj/base/tsconfig.json",
        r#"{ "compilerOptions": { "outDir": "${configDir}/out" } }"#,
    );
    fs.insert_file(
            "/proj/tsconfig.json",
            r#"{ "extends": "./base/tsconfig.json", "compilerOptions": { "rootDir": "${configDir}/src" } }"#,
        );
    let parsed = get_parsed_command_line_of_config_file(
        "/proj/tsconfig.json",
        &CompilerOptions::default(),
        "/proj",
        &fs,
    );

    assert_eq!(
        parsed.compiler_options.out_dir, "/proj/base/out",
        "extended config's ${{configDir}} should resolve to extended config's dir"
    );

    assert_eq!(
        parsed.compiler_options.root_dir, "/proj/src",
        "own config's ${{configDir}} should resolve to own config's dir"
    );
}

#[test]
fn test_config_dir_not_substituted_for_non_prefix() {
    let fs = InMemoryFS::new();
    fs.insert_dir("/proj");
    fs.insert_file(
        "/proj/tsconfig.json",
        r#"{ "compilerOptions": { "outDir": "prefix/${configDir}/out" } }"#,
    );
    let parsed = get_parsed_command_line_of_config_file(
        "/proj/tsconfig.json",
        &CompilerOptions::default(),
        "/proj",
        &fs,
    );

    assert!(
        parsed.compiler_options.out_dir.contains("configDir"),
        "embedded ${{configDir}} should not be substituted, got {}",
        parsed.compiler_options.out_dir
    );
}

#[test]
fn test_extends_inherited_include_path_rewriting() {
    let fs = InMemoryFS::new();
    fs.insert_dir("/proj");
    fs.insert_dir("/proj/base");
    fs.insert_dir("/proj/base/src");
    fs.insert_file("/proj/base/src/a.ts", "");
    fs.insert_file("/proj/base/tsconfig.json", r#"{ "include": ["src/**/*"] }"#);
    fs.insert_file(
        "/proj/tsconfig.json",
        r#"{ "extends": "./base/tsconfig.json" }"#,
    );
    let parsed = get_parsed_command_line_of_config_file(
        "/proj/tsconfig.json",
        &CompilerOptions::default(),
        "/proj",
        &fs,
    );
    assert!(
        parsed.file_names.iter().any(|f| f == "/proj/base/src/a.ts"),
        "expected /proj/base/src/a.ts in file_names (relative include rewritten), got {:?}",
        parsed.file_names
    );
}

#[test]
fn test_extends_inherited_include_absolute_not_rewritten() {
    let fs = InMemoryFS::new();
    fs.insert_dir("/proj");
    fs.insert_dir("/proj/base");
    fs.insert_dir("/shared");
    fs.insert_file("/shared/a.ts", "");
    fs.insert_file(
        "/proj/base/tsconfig.json",
        r#"{ "include": ["/shared/**/*"] }"#,
    );
    fs.insert_file(
        "/proj/tsconfig.json",
        r#"{ "extends": "./base/tsconfig.json" }"#,
    );
    let parsed = get_parsed_command_line_of_config_file(
        "/proj/tsconfig.json",
        &CompilerOptions::default(),
        "/proj",
        &fs,
    );
    assert!(
        parsed.file_names.iter().any(|f| f == "/shared/a.ts"),
        "expected /shared/a.ts in file_names (absolute include not rewritten), got {:?}",
        parsed.file_names
    );
}

#[test]
fn test_extends_inherited_include_config_dir_not_rewritten() {
    let fs = InMemoryFS::new();
    fs.insert_dir("/proj");
    fs.insert_dir("/proj/base");
    fs.insert_dir("/proj/src");
    fs.insert_file("/proj/src/a.ts", "");
    fs.insert_file(
        "/proj/base/tsconfig.json",
        r#"{ "include": ["${configDir}/src"] }"#,
    );
    fs.insert_file(
        "/proj/tsconfig.json",
        r#"{ "extends": "./base/tsconfig.json" }"#,
    );
    let parsed = get_parsed_command_line_of_config_file(
        "/proj/tsconfig.json",
        &CompilerOptions::default(),
        "/proj",
        &fs,
    );

    assert!(
        parsed.file_names.iter().any(|f| f == "/proj/src/a.ts"),
        "expected /proj/src/a.ts in file_names (${{configDir}} resolved against own dir), got {:?}",
        parsed.file_names
    );
}

#[test]
fn test_extends_inherited_exclude_path_rewriting() {
    let fs = InMemoryFS::new();
    fs.insert_dir("/proj");
    fs.insert_dir("/proj/base");
    fs.insert_dir("/proj/base/src");
    fs.insert_dir("/proj/base/excluded");
    fs.insert_file("/proj/base/src/a.ts", "");
    fs.insert_file("/proj/base/excluded/b.ts", "");
    fs.insert_file(
        "/proj/base/tsconfig.json",
        r#"{ "include": ["src/**/*"], "exclude": ["excluded"] }"#,
    );
    fs.insert_file(
        "/proj/tsconfig.json",
        r#"{ "extends": "./base/tsconfig.json" }"#,
    );
    let parsed = get_parsed_command_line_of_config_file(
        "/proj/tsconfig.json",
        &CompilerOptions::default(),
        "/proj",
        &fs,
    );

    assert!(
        parsed.file_names.iter().any(|f| f == "/proj/base/src/a.ts"),
        "expected /proj/base/src/a.ts in file_names, got {:?}",
        parsed.file_names
    );

    assert!(
        !parsed.file_names.iter().any(|f| f.contains("excluded")),
        "expected excluded/ files to be excluded, got {:?}",
        parsed.file_names
    );
}

#[test]
fn test_extends_inherited_files_path_rewriting() {
    let fs = InMemoryFS::new();
    fs.insert_dir("/proj");
    fs.insert_dir("/proj/base");
    fs.insert_dir("/proj/base/src");
    fs.insert_file("/proj/base/src/main.ts", "");
    fs.insert_file(
        "/proj/base/tsconfig.json",
        r#"{ "files": ["src/main.ts"] }"#,
    );
    fs.insert_file(
        "/proj/tsconfig.json",
        r#"{ "extends": "./base/tsconfig.json" }"#,
    );
    let parsed = get_parsed_command_line_of_config_file(
        "/proj/tsconfig.json",
        &CompilerOptions::default(),
        "/proj",
        &fs,
    );

    assert!(
        parsed
            .file_names
            .iter()
            .any(|f| f == "/proj/base/src/main.ts"),
        "expected /proj/base/src/main.ts in file_names, got {:?}",
        parsed.file_names
    );
}

#[test]
fn test_extends_own_include_overrides_inherited() {
    let fs = InMemoryFS::new();
    fs.insert_dir("/proj");
    fs.insert_dir("/proj/base");
    fs.insert_dir("/proj/own_src");
    fs.insert_file("/proj/own_src/a.ts", "");
    fs.insert_dir("/proj/base/src");
    fs.insert_file("/proj/base/src/b.ts", "");
    fs.insert_file("/proj/base/tsconfig.json", r#"{ "include": ["src/**/*"] }"#);
    fs.insert_file(
        "/proj/tsconfig.json",
        r#"{ "extends": "./base/tsconfig.json", "include": ["own_src/**/*"] }"#,
    );
    let parsed = get_parsed_command_line_of_config_file(
        "/proj/tsconfig.json",
        &CompilerOptions::default(),
        "/proj",
        &fs,
    );

    assert!(
        parsed.file_names.iter().any(|f| f == "/proj/own_src/a.ts"),
        "expected /proj/own_src/a.ts in file_names, got {:?}",
        parsed.file_names
    );
    assert!(
        !parsed.file_names.iter().any(|f| f == "/proj/base/src/b.ts"),
        "expected /proj/base/src/b.ts NOT in file_names (own include overrides), got {:?}",
        parsed.file_names
    );
}

#[test]
fn test_extends_null_clears_inherited_tristate() {
    let fs = InMemoryFS::new();
    fs.insert_dir("/proj");
    fs.insert_file(
        "/proj/base.json",
        r#"{ "compilerOptions": { "strict": true } }"#,
    );
    fs.insert_file(
        "/proj/tsconfig.json",
        r#"{ "extends": "./base.json", "compilerOptions": { "strict": null } }"#,
    );
    let parsed = get_parsed_command_line_of_config_file(
        "/proj/tsconfig.json",
        &CompilerOptions::default(),
        "/proj",
        &fs,
    );
    assert!(
        parsed.compiler_options.strict.is_unknown(),
        "expected strict=null to clear inherited strict=true, got {:?}",
        parsed.compiler_options.strict
    );
}

#[test]
fn test_extends_null_clears_inherited_string_field() {
    let fs = InMemoryFS::new();
    fs.insert_dir("/proj");
    fs.insert_file(
        "/proj/base.json",
        r#"{ "compilerOptions": { "outDir": "./dist" } }"#,
    );
    fs.insert_file(
        "/proj/tsconfig.json",
        r#"{ "extends": "./base.json", "compilerOptions": { "outDir": null } }"#,
    );
    let parsed = get_parsed_command_line_of_config_file(
        "/proj/tsconfig.json",
        &CompilerOptions::default(),
        "/proj",
        &fs,
    );
    assert!(
        parsed.compiler_options.out_dir.is_empty(),
        "expected outDir=null to clear inherited outDir, got {:?}",
        parsed.compiler_options.out_dir
    );
}

#[test]
fn test_extends_null_clears_inherited_enum_field() {
    let fs = InMemoryFS::new();
    fs.insert_dir("/proj");
    fs.insert_file(
        "/proj/base.json",
        r#"{ "compilerOptions": { "target": "ES2020" } }"#,
    );
    fs.insert_file(
        "/proj/tsconfig.json",
        r#"{ "extends": "./base.json", "compilerOptions": { "target": null } }"#,
    );
    let parsed = get_parsed_command_line_of_config_file(
        "/proj/tsconfig.json",
        &CompilerOptions::default(),
        "/proj",
        &fs,
    );
    assert_eq!(
        parsed.compiler_options.target,
        ScriptTarget::None,
        "expected target=null to clear inherited target=ES2020, got {:?}",
        parsed.compiler_options.target
    );
}

#[test]
fn test_extends_null_does_not_override_command_line() {
    let fs = InMemoryFS::new();
    fs.insert_dir("/proj");
    fs.insert_file(
        "/proj/base.json",
        r#"{ "compilerOptions": { "strict": true } }"#,
    );
    fs.insert_file(
        "/proj/tsconfig.json",
        r#"{ "extends": "./base.json", "compilerOptions": { "strict": null } }"#,
    );
    let mut base = CompilerOptions::default();
    base.strict = crate::core::tristate::Tristate::True;
    let parsed = get_parsed_command_line_of_config_file("/proj/tsconfig.json", &base, "/proj", &fs);
    assert!(
        parsed.compiler_options.strict.is_true(),
        "expected command-line strict=true to survive own strict=null, got {:?}",
        parsed.compiler_options.strict
    );
}

#[test]
fn test_extends_null_only_clears_specified_field() {
    let fs = InMemoryFS::new();
    fs.insert_dir("/proj");
    fs.insert_file(
        "/proj/base.json",
        r#"{ "compilerOptions": { "strict": true, "noImplicitAny": true } }"#,
    );
    fs.insert_file(
        "/proj/tsconfig.json",
        r#"{ "extends": "./base.json", "compilerOptions": { "strict": null } }"#,
    );
    let parsed = get_parsed_command_line_of_config_file(
        "/proj/tsconfig.json",
        &CompilerOptions::default(),
        "/proj",
        &fs,
    );
    assert!(
        parsed.compiler_options.strict.is_unknown(),
        "expected strict=null to clear inherited strict, got {:?}",
        parsed.compiler_options.strict
    );
    assert!(
        parsed.compiler_options.no_implicit_any.is_true(),
        "expected noImplicitAny to be inherited (not nulled), got {:?}",
        parsed.compiler_options.no_implicit_any
    );
}

#[test]
fn test_extends_null_with_multiple_fields() {
    let fs = InMemoryFS::new();
    fs.insert_dir("/proj");
    fs.insert_file(
        "/proj/base.json",
        r#"{ "compilerOptions": { "strict": true, "outDir": "./dist", "target": "ES2020" } }"#,
    );
    fs.insert_file(
            "/proj/tsconfig.json",
            r#"{ "extends": "./base.json", "compilerOptions": { "strict": null, "outDir": null, "target": null } }"#,
        );
    let parsed = get_parsed_command_line_of_config_file(
        "/proj/tsconfig.json",
        &CompilerOptions::default(),
        "/proj",
        &fs,
    );
    assert!(
        parsed.compiler_options.strict.is_unknown(),
        "expected strict=null to clear, got {:?}",
        parsed.compiler_options.strict
    );
    assert!(
        parsed.compiler_options.out_dir.is_empty(),
        "expected outDir=null to clear, got {:?}",
        parsed.compiler_options.out_dir
    );
    assert_eq!(
        parsed.compiler_options.target,
        ScriptTarget::None,
        "expected target=null to clear, got {:?}",
        parsed.compiler_options.target
    );
}

#[test]
fn test_extends_diamond_inheritance() {
    let fs = InMemoryFS::new();
    fs.insert_dir("/proj");
    fs.insert_file(
        "/proj/d.json",
        r#"{ "compilerOptions": { "strict": true, "noImplicitAny": true } }"#,
    );
    fs.insert_file("/proj/b.json", r#"{ "extends": "./d.json" }"#);
    fs.insert_file("/proj/c.json", r#"{ "extends": "./d.json" }"#);
    fs.insert_file(
        "/proj/tsconfig.json",
        r#"{ "extends": ["./b.json", "./c.json"] }"#,
    );
    let parsed = get_parsed_command_line_of_config_file(
        "/proj/tsconfig.json",
        &CompilerOptions::default(),
        "/proj",
        &fs,
    );

    assert!(
        parsed.compiler_options.strict.is_true(),
        "expected strict=true from diamond D, got {:?}",
        parsed.compiler_options.strict
    );
    assert!(
        parsed.compiler_options.no_implicit_any.is_true(),
        "expected noImplicitAny=true from diamond D, got {:?}",
        parsed.compiler_options.no_implicit_any
    );
}

#[test]
fn test_extends_diamond_no_duplicate_errors() {
    let fs = InMemoryFS::new();
    fs.insert_dir("/proj");

    fs.insert_file(
        "/proj/d.json",
        r#"{ "compilerOptions": { "strict": true, "Strict": true } }"#,
    );
    fs.insert_file("/proj/b.json", r#"{ "extends": "./d.json" }"#);
    fs.insert_file("/proj/c.json", r#"{ "extends": "./d.json" }"#);
    fs.insert_file(
        "/proj/tsconfig.json",
        r#"{ "extends": ["./b.json", "./c.json"] }"#,
    );
    let parsed = get_parsed_command_line_of_config_file(
        "/proj/tsconfig.json",
        &CompilerOptions::default(),
        "/proj",
        &fs,
    );

    let ts5025_count = parsed.errors.iter().filter(|d| d.code == 5025).count();
    assert_eq!(
        ts5025_count, 2,
        "expected exactly 2 TS5025 errors (D via B and C), got {}: {:?}",
        ts5025_count, parsed.errors
    );
}

#[test]
fn test_extends_cache_cycle_not_cached() {
    let fs = InMemoryFS::new();
    fs.insert_dir("/proj");
    fs.insert_file("/proj/a.json", r#"{ "extends": "./b.json" }"#);
    fs.insert_file("/proj/b.json", r#"{ "extends": "./a.json" }"#);
    fs.insert_file("/proj/tsconfig.json", r#"{ "extends": "./a.json" }"#);
    let parsed = get_parsed_command_line_of_config_file(
        "/proj/tsconfig.json",
        &CompilerOptions::default(),
        "/proj",
        &fs,
    );

    let has_cycle = parsed.errors.iter().any(|d| d.code == 18000);
    assert!(
        has_cycle,
        "expected TS18000 circularity error, got errors: {:?}",
        parsed.errors
    );
}

#[test]
fn test_extends_bare_specifier_file_form() {
    let fs = InMemoryFS::new();
    fs.insert_dir("/proj");
    fs.insert_dir("/proj/node_modules");
    fs.insert_file(
        "/proj/node_modules/tsconfig-base.json",
        r#"{ "compilerOptions": { "strict": true } }"#,
    );
    fs.insert_file("/proj/tsconfig.json", r#"{ "extends": "tsconfig-base" }"#);
    let parsed = get_parsed_command_line_of_config_file(
        "/proj/tsconfig.json",
        &CompilerOptions::default(),
        "/proj",
        &fs,
    );
    assert!(
        parsed.compiler_options.strict.is_true(),
        "expected strict=true from node_modules/tsconfig-base.json, got {:?}",
        parsed.compiler_options.strict
    );
}

#[test]
fn test_extends_bare_specifier_directory_form() {
    let fs = InMemoryFS::new();
    fs.insert_dir("/proj");
    fs.insert_dir("/proj/node_modules");
    fs.insert_dir("/proj/node_modules/tsconfig-base");
    fs.insert_file(
        "/proj/node_modules/tsconfig-base/tsconfig.json",
        r#"{ "compilerOptions": { "noImplicitAny": true } }"#,
    );
    fs.insert_file("/proj/tsconfig.json", r#"{ "extends": "tsconfig-base" }"#);
    let parsed = get_parsed_command_line_of_config_file(
        "/proj/tsconfig.json",
        &CompilerOptions::default(),
        "/proj",
        &fs,
    );
    assert!(
        parsed.compiler_options.no_implicit_any.is_true(),
        "expected noImplicitAny=true from node_modules/tsconfig-base/tsconfig.json, got {:?}",
        parsed.compiler_options.no_implicit_any
    );
}

#[test]
fn test_extends_bare_specifier_package_json_tsconfig_field() {
    let fs = InMemoryFS::new();
    fs.insert_dir("/proj");
    fs.insert_dir("/proj/node_modules");
    fs.insert_dir("/proj/node_modules/tsconfig-base");
    fs.insert_file(
        "/proj/node_modules/tsconfig-base/package.json",
        r#"{ "name": "tsconfig-base", "tsconfig": "my-base.json" }"#,
    );
    fs.insert_file(
        "/proj/node_modules/tsconfig-base/my-base.json",
        r#"{ "compilerOptions": { "strict": true, "noImplicitThis": true } }"#,
    );
    fs.insert_file("/proj/tsconfig.json", r#"{ "extends": "tsconfig-base" }"#);
    let parsed = get_parsed_command_line_of_config_file(
        "/proj/tsconfig.json",
        &CompilerOptions::default(),
        "/proj",
        &fs,
    );
    assert!(
        parsed.compiler_options.strict.is_true(),
        "expected strict=true from package.json tsconfig field, got {:?}",
        parsed.compiler_options.strict
    );
    assert!(
        parsed.compiler_options.no_implicit_this.is_true(),
        "expected noImplicitThis=true from package.json tsconfig field, got {:?}",
        parsed.compiler_options.no_implicit_this
    );
}

#[test]
fn test_extends_bare_specifier_scoped_package() {
    let fs = InMemoryFS::new();
    fs.insert_dir("/proj");
    fs.insert_dir("/proj/node_modules");
    fs.insert_dir("/proj/node_modules/@scope");
    fs.insert_dir("/proj/node_modules/@scope/tsconfig-base");
    fs.insert_file(
        "/proj/node_modules/@scope/tsconfig-base/tsconfig.json",
        r#"{ "compilerOptions": { "strictNullChecks": true } }"#,
    );
    fs.insert_file(
        "/proj/tsconfig.json",
        r#"{ "extends": "@scope/tsconfig-base" }"#,
    );
    let parsed = get_parsed_command_line_of_config_file(
        "/proj/tsconfig.json",
        &CompilerOptions::default(),
        "/proj",
        &fs,
    );
    assert!(
        parsed.compiler_options.strict_null_checks.is_true(),
        "expected strictNullChecks=true from @scope/tsconfig-base, got {:?}",
        parsed.compiler_options.strict_null_checks
    );
}

#[test]
fn test_extends_bare_specifier_ancestor_walk() {
    let fs = InMemoryFS::new();
    fs.insert_dir("/proj");
    fs.insert_dir("/proj/node_modules");
    fs.insert_dir("/proj/node_modules/tsconfig-base");
    fs.insert_file(
        "/proj/node_modules/tsconfig-base/tsconfig.json",
        r#"{ "compilerOptions": { "strict": true } }"#,
    );
    fs.insert_dir("/proj/packages");
    fs.insert_dir("/proj/packages/foo");
    fs.insert_file(
        "/proj/packages/foo/tsconfig.json",
        r#"{ "extends": "tsconfig-base" }"#,
    );
    let parsed = get_parsed_command_line_of_config_file(
        "/proj/packages/foo/tsconfig.json",
        &CompilerOptions::default(),
        "/proj/packages/foo",
        &fs,
    );
    assert!(
        parsed.compiler_options.strict.is_true(),
        "expected strict=true from ancestor node_modules, got {:?}",
        parsed.compiler_options.strict
    );
}

#[test]
fn test_extends_bare_specifier_not_found() {
    let fs = InMemoryFS::new();
    fs.insert_dir("/proj");
    fs.insert_file(
        "/proj/tsconfig.json",
        r#"{ "extends": "nonexistent-config", "compilerOptions": { "target": "ES2020" } }"#,
    );
    let parsed = get_parsed_command_line_of_config_file(
        "/proj/tsconfig.json",
        &CompilerOptions::default(),
        "/proj",
        &fs,
    );

    assert_eq!(
        parsed.compiler_options.target,
        ScriptTarget::ES2020,
        "expected own config target to be applied"
    );

    assert!(
        !parsed.compiler_options.strict.is_true(),
        "expected strict=false (no extended config found), got {:?}",
        parsed.compiler_options.strict
    );
}
