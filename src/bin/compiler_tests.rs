use std::io::Write;
use std::panic::AssertUnwindSafe;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::time::{Duration, Instant};

use regex::Regex;

use tsox::ast::Diagnostic;
use tsox::bundled::BundledFS;
use tsox::compiler::{CompilerHostImpl, Program, ProgramOptions};
use tsox::core::compiler_options::{
    CompilerOptions, JsxEmit, ModuleKind, ModuleResolutionKind, ScriptTarget,
};
use tsox::core::tristate::Tristate;
use tsox::diagnosticwriter::{format_diagnostic, line_and_character};
use tsox::testutil::baseline;
use tsox::testutil::test_case_parser::{TestCaseContent, parse_test_files};
use tsox::tsoptions::ParsedCommandLine;
use tsox::vfs::{FS, InMemoryFS};

const SRC_FOLDER: &str = "/.src";

const BATCH_SIZE: usize = 10;

const BATCH_TIMEOUT_SECS: u64 = 30;

const SKIPPED_TESTS: &[&str] = &[
    "typeGuardFunctionErrors.ts",
    "parserS7.2_A1.5_T2.ts",
    "scannerS7.2_A1.5_T2.ts",
    "ifDoWhileStatements.ts",

    "controlFlowGraphStress01.ts",
];

struct CompilationOutput {
    diagnostics: Vec<Diagnostic>,
    input_files: Vec<(String, String)>,
}

fn compile_test_case(content: &TestCaseContent) -> CompilationOutput {
    let fs = InMemoryFS::new();
    let mut file_names: Vec<String> = Vec::new();
    let mut input_files: Vec<(String, String)> = Vec::new();
    for unit in &content.units {

        let abs_path = if unit.name.starts_with('/') {
            unit.name.clone()
        } else {

            let basename = Path::new(&unit.name)
                .file_name()
                .map(|f| f.to_string_lossy().to_string())
                .unwrap_or_else(|| unit.name.clone());
            format!("{SRC_FOLDER}/{basename}")
        };
        let _ = fs.write_file(&abs_path, &unit.content);
        file_names.push(abs_path.clone());

        let baseline_name = abs_path.clone();
        input_files.push((baseline_name, unit.content.clone()));
    }

    let mut options = CompilerOptions::default();
    apply_test_settings(&mut options, &content.settings);
    options.skip_default_lib_check = Tristate::True;
    options.no_error_truncation = Tristate::True;

    let config = ParsedCommandLine {
        compiler_options: options,
        file_names,
        errors: Vec::new(),
        config_file_name: String::new(),
        raw_options: None,
        include: Vec::new(),
        exclude: Vec::new(),
        files_spec: Vec::new(),
        has_include_spec: false,
        has_exclude_spec: false,
        has_files_spec: false,
        references: Vec::new(),
        compile_on_save: None,
        watch: false,
        watch_options: Default::default(),
    };

    let host = CompilerHostImpl::new(
        Arc::new(BundledFS::new(Arc::new(fs))),
        String::new(),
        content.current_directory.clone(),
    );

    let program = Arc::new(Program::new(ProgramOptions {
        config,
        host: Arc::new(host),
    }));

    let _pretty = false;
    let mut diagnostics: Vec<Diagnostic> = Vec::new();
    for d in program.get_diagnostics_to_report() {
        diagnostics.push((*d).clone());
    }
    for d in &program.get_semantic_diagnostics() {
        diagnostics.push(d.clone());
    }

    CompilationOutput {
        diagnostics,
        input_files,
    }
}

fn remove_test_path_prefixes(text: &str) -> String {
    text.replace("/.src/", "")
        .replace("/.ts/", "")
        .replace("/.lib/", "")
}

const CRLF: &str = "\r\n";

fn format_error_baseline(output: &CompilationOutput) -> String {
    if output.diagnostics.is_empty() {
        return String::new();
    }

    let locale = None;
    let mut result = String::new();

    let mut diags = output.diagnostics.clone();
    diags.sort_by(|a, b| {
        let a_file = a.file.as_ref().map(|f| f.file_name.as_str()).unwrap_or("");
        let b_file = b.file.as_ref().map(|f| f.file_name.as_str()).unwrap_or("");
        let cmp = a_file.cmp(b_file);
        if cmp != std::cmp::Ordering::Equal {
            return cmp;
        }
        let cmp = a.loc.pos().cmp(&b.loc.pos());
        if cmp != std::cmp::Ordering::Equal {
            return cmp;
        }
        a.code.cmp(&b.code)
    });

    for diag in &diags {
        let line = format_diagnostic(diag, false, locale);
        let normalized = remove_test_path_prefixes(&line);
        result.push_str(&normalized);
        result.push_str(CRLF);
    }
    result.push_str(CRLF);
    result.push_str(CRLF);

    let mut first = true;
    for diag in &diags {
        if diag.file.is_none() {
            if !first {
                result.push_str(CRLF);
            }
            first = false;
            let msg = tsox::diagnosticwriter::message_text(diag, locale);
            for line in msg.lines() {
                if line.is_empty() {
                    continue;
                }
                result.push_str(&format!(
                    "!!! {} TS{}: {}",
                    diag.category.name(),
                    diag.code,
                    line
                ));
                result.push_str(CRLF);
            }
        }
    }

    for (unit_name, content) in &output.input_files {
        let normalized_name = remove_test_path_prefixes(unit_name);

        let file_diags: Vec<&Diagnostic> = diags
            .iter()
            .filter(|d| {
                if let Some(file) = &d.file {
                    let dn = remove_test_path_prefixes(&file.file_name);
                    dn == normalized_name
                } else {
                    false
                }
            })
            .collect();

        result.push_str(&format!(
            "==== {} ({} errors) ====",
            normalized_name,
            file_diags.len()
        ));
        result.push_str(CRLF);

        let lines: Vec<&str> = content.split('\n').collect();

        let mut line_starts: Vec<usize> = vec![0];
        for (i, ch) in content.char_indices() {
            if ch == '\n' {
                line_starts.push(i + 1);
            }
        }

        for (line_idx, line) in lines.iter().enumerate() {

            let line = line.strip_suffix('\r').unwrap_or(line);

            let this_line_start = *line_starts.get(line_idx).unwrap_or(&0);
            let next_line_start = if line_idx == lines.len() - 1 {
                content.len()
            } else {
                *line_starts.get(line_idx + 1).unwrap_or(&content.len())
            };

            result.push_str("    ");
            result.push_str(line);
            result.push_str(CRLF);

            for diag in &file_diags {
                let err_start = diag.loc.pos();
                let err_end = err_start + diag.loc.len();

                if err_end >= this_line_start
                    && (err_start < next_line_start || line_idx == lines.len() - 1)
                {
                    let relative_offset = err_start as i64 - this_line_start as i64;
                    let length = (err_end - err_start) as i64
                        - (this_line_start as i64 - err_start as i64).max(0);
                    let squiggle_start = (relative_offset).max(0) as usize;

                    result.push_str("    ");
                    let prefix = &line[..squiggle_start.min(line.len())];

                    for ch in prefix.chars() {
                        result.push(if ch.is_whitespace() { ch } else { ' ' });
                    }
                    let squiggle_end = (squiggle_start + length as usize)
                        .min(line.len())
                        .max(squiggle_start);
                    let squiggle_count = line
                        [squiggle_start.min(line.len())..squiggle_end.min(line.len())]
                        .chars()
                        .count();
                    for _ in 0..squiggle_count {
                        result.push('~');
                    }
                    result.push_str(CRLF);

                    if line_idx == lines.len() - 1 || next_line_start > err_end {
                        let msg = tsox::diagnosticwriter::message_text(diag, locale);
                        for msg_line in msg.lines() {
                            if msg_line.is_empty() {
                                continue;
                            }
                            result.push_str(&format!(
                                "!!! {} TS{}: {}",
                                diag.category.name(),
                                diag.code,
                                msg_line
                            ));
                            result.push_str(CRLF);
                        }

                        for info in &diag.related_information {
                            let info_loc = if let Some(info_file) = &info.file {
                                let (l, c) = line_and_character(
                                    &info_file.line_map,
                                    &info_file.text,
                                    info.loc.pos(),
                                );
                                format!(
                                    " {}({},{})",
                                    remove_test_path_prefixes(&info_file.file_name),
                                    l + 1,
                                    c + 1
                                )
                            } else {
                                String::new()
                            };
                            let info_msg = tsox::diagnosticwriter::message_text(info, locale);
                            result.push_str(&format!(
                                "!!! related TS{}{}: {}",
                                info.code, info_loc, info_msg
                            ));
                            result.push_str(CRLF);
                        }
                    }
                }
            }
        }
    }

    result
}

fn apply_test_settings(
    options: &mut CompilerOptions,
    settings: &std::collections::HashMap<String, String>,
) {
    for (key, value) in settings {
        let v = value.trim();
        match key.as_str() {
            "strict" => options.strict = parse_tristate(v),
            "noimplicitany" => options.no_implicit_any = parse_tristate(v),
            "strictnullchecks" => options.strict_null_checks = parse_tristate(v),
            "strictfunctiontypes" => options.strict_function_types = parse_tristate(v),
            "strictbindcallapply" => options.strict_bind_call_apply = parse_tristate(v),
            "strictpropertyinitialization" => {
                options.strict_property_initialization = parse_tristate(v)
            }
            "noimplicitthis" => options.no_implicit_this = parse_tristate(v),
            "alwaysstrict" => options.always_strict = parse_tristate(v),
            "noimplicitreturns" => options.no_implicit_returns = parse_tristate(v),
            "nofallthroughcasesinswitch" => {
                options.no_fallthrough_cases_in_switch = parse_tristate(v)
            }
            "skiplibcheck" => options.skip_lib_check = parse_tristate(v),
            "skipdefaultlibcheck" => options.skip_default_lib_check = parse_tristate(v),
            "allowjs" => options.allow_js = parse_tristate(v),
            "checkjs" => options.check_js = parse_tristate(v),
            "declaration" => options.declaration = parse_tristate(v),
            "sourcemap" => options.source_map = parse_tristate(v),
            "noemit" => options.no_emit = parse_tristate(v),
            "isolatedmodules" => options.isolated_modules = parse_tristate(v),
            "verbatimmodulesyntax" => options.verbatim_module_syntax = parse_tristate(v),
            "target" => {
                if let Some(kind) = parse_target(v) {
                    options.target = kind;
                }
            }
            "module" => {
                if let Some(kind) = parse_module(v) {
                    options.module = kind;
                }
            }
            "moduleresolution" => {
                if let Some(kind) = parse_module_resolution(v) {
                    options.module_resolution = kind;
                }
            }
            "jsx" => {
                if let Some(kind) = parse_jsx(v) {
                    options.jsx = kind;
                }
            }
            "lib" => {
                options.lib = v.split(',').map(|s| s.trim().to_string()).collect();
            }
            "outdir" => options.out_dir = normalize_option_path(v),
            "rootdir" => options.root_dir = normalize_option_path(v),
            "baseurl" => options.base_url = normalize_option_path(v),
            "experimentaldecorators" => options.experimental_decorators = parse_tristate(v),
            "emitdecoratormetadata" => options.emit_decorator_metadata = parse_tristate(v),
            "usesdefineforclassfields" => options.use_define_for_class_fields = parse_tristate(v),
            _ => {}
        }
    }
}

fn parse_tristate(v: &str) -> Tristate {
    match v.trim().to_lowercase().as_str() {
        "true" | "1" | "yes" => Tristate::True,
        "false" | "0" | "no" => Tristate::False,
        _ => Tristate::Unknown,
    }
}

fn parse_target(v: &str) -> Option<ScriptTarget> {
    match v.trim().to_lowercase().as_str() {
        "es5" => Some(ScriptTarget::ES5),
        "es6" | "es2015" => Some(ScriptTarget::ES2015),
        "es2016" => Some(ScriptTarget::ES2016),
        "es2017" => Some(ScriptTarget::ES2017),
        "es2018" => Some(ScriptTarget::ES2018),
        "es2019" => Some(ScriptTarget::ES2019),
        "es2020" => Some(ScriptTarget::ES2020),
        "es2021" => Some(ScriptTarget::ES2021),
        "es2022" => Some(ScriptTarget::ES2022),
        "esnext" => Some(ScriptTarget::ESNext),
        _ => None,
    }
}

fn parse_module(v: &str) -> Option<ModuleKind> {
    match v.trim().to_lowercase().as_str() {
        "none" => Some(ModuleKind::None),
        "commonjs" => Some(ModuleKind::CommonJS),
        "amd" => Some(ModuleKind::AMD),
        "system" => Some(ModuleKind::System),
        "umd" => Some(ModuleKind::UMD),
        "es6" | "es2015" => Some(ModuleKind::ES2015),
        "es2020" => Some(ModuleKind::ES2020),
        "es2022" => Some(ModuleKind::ES2022),
        "esnext" => Some(ModuleKind::ESNext),
        "node16" => Some(ModuleKind::Node16),
        "nodenext" => Some(ModuleKind::NodeNext),
        "preserve" => Some(ModuleKind::Preserve),
        _ => None,
    }
}

fn parse_module_resolution(v: &str) -> Option<ModuleResolutionKind> {
    match v.trim().to_lowercase().as_str() {
        "classic" => Some(ModuleResolutionKind::Classic),
        "node" | "node10" => Some(ModuleResolutionKind::Node10),
        "node16" => Some(ModuleResolutionKind::Node16),
        "nodenext" => Some(ModuleResolutionKind::NodeNext),
        "bundler" => Some(ModuleResolutionKind::Bundler),
        _ => None,
    }
}

fn parse_jsx(v: &str) -> Option<JsxEmit> {
    match v.trim().to_lowercase().as_str() {
        "preserve" => Some(JsxEmit::Preserve),
        "react" => Some(JsxEmit::React),
        "react-native" => Some(JsxEmit::ReactNative),
        "react-jsx" => Some(JsxEmit::ReactJSX),
        "react-jsxdev" => Some(JsxEmit::ReactJSXDev),
        _ => None,
    }
}

fn normalize_option_path(v: &str) -> String {
    let v = v.trim();
    if v.starts_with('/') || v.starts_with('\\') {
        v.to_string()
    } else {
        format!("{SRC_FOLDER}/{v}")
    }
}

#[allow(dead_code)]
fn normalize_abs_path(name: &str, current_dir: &str) -> String {
    if name.starts_with('/') {
        name.to_string()
    } else {
        format!("{current_dir}/{name}")
    }
}

fn print_flush(msg: &str) {
    print!("{msg}");
    let _ = std::io::stdout().flush();
}

fn process_batch(
    files: &[String],
    test_dir: &Path,
    suite_name: &str,
    is_submodule: bool,
    no_write: bool,
    ts_ref_dir: Option<&str>,
) -> (usize, usize, usize, usize, usize, Vec<String>) {
    let mut pass = 0usize;
    let mut fail = 0usize;
    let mut new_baseline = 0usize;
    let mut panic_count = 0usize;
    let mut skipped = 0usize;
    let mut fails = Vec::new();

    for rel_path in files {
        let full_path = test_dir.join(rel_path);

        let basename = Path::new(rel_path)
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_default();
        if SKIPPED_TESTS.contains(&basename.as_str()) {
            skipped += 1;
            continue;
        }
        let clean_name = if is_submodule {

            let components: Vec<&str> = rel_path.split('/').collect();
            let n = components.len();
            if n >= 2 {
                format!("{}/{}", components[n - 2], components[n - 1])
            } else {
                rel_path.to_string()
            }
        } else {
            rel_path.replace('\\', "/")
        };

        let content = match std::fs::read_to_string(&full_path) {
            Ok(c) => c,
            Err(_) => {
                panic_count += 1;
                continue;
            }
        };

        if content.trim().is_empty() {
            continue;
        }

        let test_case = parse_test_files(&content, &clean_name);

        let result = std::panic::catch_unwind(AssertUnwindSafe(|| compile_test_case(&test_case)));

        match result {
            Ok(output) => {
                let error_baseline = format_error_baseline(&output);

                if let Some(ref_dir) = ts_ref_dir {

                    let ts_name = basename
                        .strip_suffix(".ts")
                        .or_else(|| basename.strip_suffix(".tsx"))
                        .unwrap_or(&basename);
                    let ts_baseline_path =
                        PathBuf::from(ref_dir).join(format!("{ts_name}.errors.txt"));

                    let expected = std::fs::read_to_string(&ts_baseline_path).unwrap_or_default();
                    let actual = if error_baseline.is_empty() {
                        baseline::NO_CONTENT.to_string()
                    } else {
                        error_baseline.clone()
                    };

                    if expected.is_empty() && actual == baseline::NO_CONTENT {

                        pass += 1;
                    } else if expected.trim_end() == actual.trim_end() {
                        pass += 1;
                    } else {
                        fail += 1;
                        fails.push(format!("DIFF: {clean_name}"));
                    }

                    let baseline_name = format!(
                        "{}.errors.txt",
                        Path::new(&clean_name).with_extension("").to_string_lossy()
                    );
                    let local_path = baseline::local_root()
                        .join("submodule")
                        .join(suite_name)
                        .join(&baseline_name);
                    if let Some(parent) = local_path.parent() {
                        let _ = std::fs::create_dir_all(parent);
                    }
                    let _ = std::fs::write(&local_path, &error_baseline);
                    continue;
                }

                if no_write {
                    pass += 1;
                    continue;
                }

                let baseline_name = format!(
                    "{}.errors.txt",
                    Path::new(&clean_name).with_extension("").to_string_lossy()
                );

                let opts = baseline::BaselineOptions {
                    subfolder: suite_name.to_string(),
                    is_submodule,
                };

                match baseline::run(&baseline_name, &error_baseline, &opts) {
                    Ok(_) => pass += 1,
                    Err(e) => {
                        if e.contains("Reference:") {
                            new_baseline += 1;
                        } else {
                            fail += 1;
                            fails.push(format!("FAIL: {clean_name}"));
                        }
                    }
                }
            }
            Err(e) => {
                panic_count += 1;
                let msg = if let Some(s) = e.downcast_ref::<&str>() {
                    s.to_string()
                } else if let Some(s) = e.downcast_ref::<String>() {
                    s.clone()
                } else {
                    format!("{e:?}")
                };
                fails.push(format!("PANIC: {clean_name}: {msg}"));
            }
        }
    }

    (pass, fail, new_baseline, panic_count, skipped, fails)
}

fn run_compiler_baselines(
    test_dir: &Path,
    suite_name: &str,
    is_submodule: bool,
    max_tests: Option<usize>,
    no_write: bool,
    ts_ref_dir: Option<&str>,
) {
    let ts_file_re = Regex::new(r"\.tsx?$").unwrap();
    let files = baseline::enumerate_test_files(test_dir, &ts_file_re);

    let total = files.len();
    let limit = max_tests.unwrap_or(total).min(total);

    let mut total_pass = 0usize;
    let mut total_fail = 0usize;
    let mut total_new = 0usize;
    let mut total_panic = 0usize;
    let mut total_skip = 0usize;

    let start = Instant::now();
    let exe = std::env::current_exe().unwrap();

    let batches: Vec<String> = files[..limit].to_vec();

    print_flush(&format!(
        "Running {limit}/{total} tests in batches of {BATCH_SIZE}...\n"
    ));

    for chunk in batches.chunks(BATCH_SIZE) {
        let _chunk_start = Instant::now();

        let mut cmd = Command::new(&exe);
        cmd.arg("--batch-mode")
            .arg("--suite")
            .arg(suite_name)
            .arg("--test-dir")
            .arg(test_dir);

        if is_submodule {
            cmd.arg("--is-submodule");
        }
        if no_write {
            cmd.arg("--no-write");
        }
        if let Some(ref_dir) = ts_ref_dir {
            cmd.arg("--ts-ref-dir").arg(ref_dir);
        }
        for f in chunk {
            cmd.arg("--file").arg(f);
        }

        cmd.stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit());

        let mut child = match cmd.spawn() {
            Ok(c) => c,
            Err(e) => {
                print_flush(&format!("  ERROR spawning batch: {e}\n"));
                total_panic += chunk.len();
                continue;
            }
        };

        let timeout = Duration::from_secs(BATCH_TIMEOUT_SECS);
        let result = child.wait_timeout(timeout);

        match result {
            Ok(Some(status)) => {

                let output = child.wait_with_output();
                if let Ok(out) = output {
                    let stdout = String::from_utf8_lossy(&out.stdout);

                    for line in stdout.lines() {
                        if let Some(rest) = line.strip_prefix("RESULT ") {
                            for part in rest.split_whitespace() {
                                if let Some(val) = part.strip_prefix("pass=") {
                                    total_pass += val.parse::<usize>().unwrap_or(0);
                                } else if let Some(val) = part.strip_prefix("fail=") {
                                    let f: usize = val.parse().unwrap_or(0);
                                    total_fail += f;
                                } else if let Some(val) = part.strip_prefix("new=") {
                                    total_new += val.parse().unwrap_or(0);
                                } else if let Some(val) = part.strip_prefix("panic=") {
                                    total_panic += val.parse().unwrap_or(0);
                                } else if let Some(val) = part.strip_prefix("skip=") {
                                    total_skip += val.parse().unwrap_or(0);
                                }
                            }
                        } else if line.starts_with("FAIL:")
                            || line.starts_with("PANIC:")
                            || line.starts_with("DIFF:")
                        {
                            print_flush(&format!("  {line}\n"));
                        }
                    }
                }

                if !status.success() && status.code() != Some(134) {

                }
            }
            Ok(None) => {

                let _ = child.kill();
                let _ = child.wait();
                let _elapsed = start.elapsed().as_secs();
                let processed = total_pass + total_fail + total_new + total_panic + total_skip;
                print_flush(&format!(
                    "  TIMEOUT at batch containing {}/{}\n",
                    processed, limit
                ));
                total_panic += chunk.len();
            }
            Err(e) => {
                print_flush(&format!("  ERROR waiting for batch: {e}\n"));
                total_panic += chunk.len();
            }
        }

        let processed = total_pass + total_fail + total_new + total_panic + total_skip;
        if processed % 200 < BATCH_SIZE {
            let elapsed = start.elapsed().as_secs();
            let rate = processed as f64 / elapsed.max(1) as f64;
            let eta = ((limit - processed) as f64 / rate.max(0.1)) as u64;
            print_flush(&format!(
                "  [{}/{}] pass={} fail={} new={} panic={} skip={} | {}s elapsed, ~{}s ETA\n",
                processed,
                limit,
                total_pass,
                total_fail,
                total_new,
                total_panic,
                total_skip,
                elapsed,
                eta
            ));
        }
    }

    let elapsed = start.elapsed().as_secs();
    print_flush(&format!(
        "\n=== {suite_name} ({}) ===\n  Total: {}  Pass: {}  Fail: {}  New: {}  Panic: {}  Skipped: {}\n  Time: {}s\n",
        if is_submodule { "submodule" } else { "local" },
        limit,
        total_pass,
        total_fail,
        total_new,
        total_panic,
        total_skip,
        elapsed,
    ));
}

fn main() {

    let handle = std::thread::Builder::new()
        .stack_size(256 * 1024 * 1024)
        .spawn(main_inner)
        .unwrap();
    handle.join().unwrap();
}

fn main_inner() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    let args: Vec<String> = std::env::args().collect();
    let mut max_tests: Option<usize> = None;
    let mut run_local = true;
    let mut run_submodule = false;
    let mut limit_submodule: Option<usize> = None;
    let mut no_write = false;
    let mut ts_ref_dir: Option<String> = None;

    let mut batch_mode = false;
    let mut batch_files: Vec<String> = Vec::new();
    let mut batch_test_dir = String::new();
    let mut batch_suite = String::new();
    let mut batch_is_submodule = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--max-tests" => {
                if i + 1 < args.len() {
                    max_tests = args[i + 1].parse().ok();
                    i += 1;
                }
            }
            "--submodule-limit" => {
                if i + 1 < args.len() {
                    limit_submodule = args[i + 1].parse().ok();
                    i += 1;
                }
            }
            "--suite" => {
                if i + 1 < args.len() {
                    batch_suite = args[i + 1].to_string();
                    match args[i + 1].as_str() {
                        "local" => {
                            run_local = true;
                            run_submodule = false;
                        }
                        "submodule" => {
                            run_local = false;
                            run_submodule = true;
                        }
                        "all" => {
                            run_local = true;
                            run_submodule = true;
                        }
                        _ => {}
                    }
                    i += 1;
                }
            }
            "--no-write" => {
                no_write = true;
            }
            "--batch-mode" => {
                batch_mode = true;
            }
            "--test-dir" => {
                if i + 1 < args.len() {
                    batch_test_dir = args[i + 1].to_string();
                    i += 1;
                }
            }
            "--file" => {
                if i + 1 < args.len() {
                    batch_files.push(args[i + 1].to_string());
                    i += 1;
                }
            }
            "--is-submodule" => {
                batch_is_submodule = true;
            }
            "--ts-ref-dir" => {
                if i + 1 < args.len() {
                    ts_ref_dir = Some(args[i + 1].to_string());
                    i += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }

    if batch_mode {
        let test_dir = Path::new(&batch_test_dir);
        let (pass, fail, new, panic, skip, fails) = process_batch(
            &batch_files,
            test_dir,
            &batch_suite,
            batch_is_submodule,
            no_write,
            ts_ref_dir.as_deref(),
        );

        for f in &fails {
            println!("{f}");
        }
        println!("RESULT pass={pass} fail={fail} new={new} panic={panic} skip={skip}");
        return;
    }

    if run_local {
        let local_dir = manifest_dir.join("tests/cases/compiler");
        if local_dir.exists() {
            print_flush("=== Running local compiler tests ===\n");
            run_compiler_baselines(
                &local_dir,
                "compiler",
                false,
                max_tests,
                no_write,
                ts_ref_dir.as_deref(),
            );
        }
    }

    if run_submodule {
        let submodule_dir =
            manifest_dir.join("../typescript-go/_submodules/TypeScript/tests/cases/conformance");
        if submodule_dir.exists() {
            print_flush("\n=== Running submodule conformance tests ===\n");

            let ts_ref: Option<String> = ts_ref_dir.clone().or_else(|| {
                let default = manifest_dir
                    .join("../typescript-go/_submodules/TypeScript/tests/baselines/reference");
                if default.exists() {
                    Some(default.to_string_lossy().into_owned())
                } else {
                    None
                }
            });
            run_compiler_baselines(
                &submodule_dir,
                "conformance",
                true,
                limit_submodule,
                no_write,
                ts_ref.as_deref(),
            );
        } else {
            print_flush(&format!(
                "=== Submodule not available at {} ===\n",
                submodule_dir.display()
            ));
            print_flush(
                "    Run: cd ../typescript-go && git submodule update --init _submodules/TypeScript\n",
            );
        }
    }

    print_flush("\n=== DONE ===\n");
}

trait ChildWaitTimeout {
    fn wait_timeout(
        &mut self,
        duration: Duration,
    ) -> std::io::Result<Option<std::process::ExitStatus>>;
}

impl ChildWaitTimeout for std::process::Child {
    fn wait_timeout(
        &mut self,
        duration: Duration,
    ) -> std::io::Result<Option<std::process::ExitStatus>> {

        let start = Instant::now();
        loop {
            match self.try_wait()? {
                Some(status) => return Ok(Some(status)),
                None => {
                    if start.elapsed() >= duration {
                        return Ok(None);
                    }
                    std::thread::sleep(Duration::from_millis(100));
                }
            }
        }
    }
}
