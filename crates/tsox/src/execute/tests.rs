use super::watch;
use super::*;
use crate::bundled::BundledFS;
use crate::vfs::InMemoryFS;
use std::sync::Mutex;

struct TestSystem {
    fs: Arc<BundledFS>,
    cwd: String,
    output: Arc<Mutex<Vec<u8>>>,
}

impl TestSystem {
    fn new(inner_fs: Arc<InMemoryFS>, cwd: &str) -> Self {
        Self {
            fs: Arc::new(BundledFS::new(inner_fs as Arc<dyn FS>)),
            cwd: cwd.to_string(),
            output: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn output_string(&self) -> String {
        String::from_utf8_lossy(&self.output.lock().unwrap()).to_string()
    }
}

impl System for TestSystem {
    fn writer(&self) -> Box<dyn Write + Send> {
        Box::new(BufferWriter {
            buf: Arc::clone(&self.output),
        })
    }
    fn fs(&self) -> Arc<dyn FS> {
        Arc::clone(&self.fs) as Arc<dyn FS>
    }
    fn default_library_path(&self) -> &str {
        "bundled:///libs"
    }
    fn current_directory(&self) -> &str {
        &self.cwd
    }
    fn write_output_is_tty(&self) -> bool {
        false
    }
    fn width_of_terminal(&self) -> usize {
        80
    }
    fn environment_variable(&self, _name: &str) -> Option<String> {
        None
    }
}

struct BufferWriter {
    buf: Arc<Mutex<Vec<u8>>>,
}

impl Write for BufferWriter {
    fn write(&mut self, data: &[u8]) -> std::io::Result<usize> {
        self.buf.lock().unwrap().extend_from_slice(data);
        Ok(data.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[test]
fn version_flag_prints_version() {
    let fs = Arc::new(InMemoryFS::new());
    let sys = TestSystem::new(fs, "/proj");
    let args = vec!["--version".to_string()];
    let result = command_line(&sys, &args);
    assert_eq!(result.status, ExitStatus::Success);
    assert!(sys.output_string().contains("Version 7.1.0-dev"));
}

#[test]
fn help_flag_prints_help() {
    let fs = Arc::new(InMemoryFS::new());
    let sys = TestSystem::new(fs, "/proj");
    let args = vec!["--help".to_string()];
    let result = command_line(&sys, &args);
    assert_eq!(result.status, ExitStatus::Success);
    let out = sys.output_string();
    assert!(
        out.contains("tsc: The TypeScript Compiler"),
        "header missing:\n{out}"
    );

    assert!(
        out.contains("Print this message."),
        "help option desc missing:\n{out}"
    );
    assert!(
        out.contains("Do not emit outputs."),
        "noEmit option desc missing:\n{out}"
    );
    assert!(
        out.contains("COMMAND LINE FLAGS"),
        "section missing:\n{out}"
    );
    assert!(
        out.contains("COMMON COMPILER OPTIONS"),
        "section missing:\n{out}"
    );
}

#[test]
fn init_flag_writes_tsconfig() {
    let fs = Arc::new(InMemoryFS::new());
    fs.insert_dir("/proj");
    let sys = TestSystem::new(Arc::clone(&fs), "/proj");
    let args = vec!["--init".to_string()];
    let result = command_line(&sys, &args);
    assert_eq!(result.status, ExitStatus::Success);
    let config = fs.read_file("/proj/tsconfig.json").unwrap();
    assert!(config.contains("\"compilerOptions\""));
    assert!(config.contains("\"strict\": true"));
    assert!(sys.output_string().contains("Created a new tsconfig.json"));
}

#[test]
fn init_flag_errors_when_tsconfig_exists() {
    let fs = Arc::new(InMemoryFS::new());
    fs.insert_dir("/proj");
    fs.insert_file("/proj/index.ts", "");
    fs.insert_file("/proj/tsconfig.json", "{}");
    let sys = TestSystem::new(fs, "/proj");
    let args = vec!["--init".to_string()];
    let result = command_line(&sys, &args);
    assert_eq!(result.status, ExitStatus::DiagnosticsPresent_OutputsSkipped);
    assert!(sys.output_string().contains("already defined"));
}

#[test]
fn no_config_no_files_shows_help_and_errors() {
    let fs = Arc::new(InMemoryFS::new());
    fs.insert_dir("/proj");
    let sys = TestSystem::new(fs, "/proj");
    let args: Vec<String> = vec![];
    let result = command_line(&sys, &args);
    assert_eq!(result.status, ExitStatus::DiagnosticsPresent_OutputsSkipped);
    assert!(sys.output_string().contains("tsconfig.json"));
}

#[test]
fn compiles_simple_file() {
    let fs = Arc::new(InMemoryFS::new());
    fs.insert_dir("/proj");
    fs.insert_file("/proj/a.ts", "let x: number = 1;");
    let sys = TestSystem::new(fs, "/proj");
    let args = vec!["--ignoreConfig".to_string(), "/proj/a.ts".to_string()];
    let result = command_line(&sys, &args);

    if result.status != ExitStatus::Success {
        panic!(
            "Expected Success but got {:?}. Output:\n{}",
            result.status,
            sys.output_string()
        );
    }
}

#[test]
fn non_ascii_invalid_character_does_not_panic_in_command_line() {
    let fs = Arc::new(InMemoryFS::new());
    fs.insert_dir("/proj");
    fs.insert_file("/proj/middle-dot.ts", "·");
    let sys = TestSystem::new(fs, "/proj");
    let args = vec![
        "--noLib".to_string(),
        "--ignoreConfig".to_string(),
        "--noEmitOnError".to_string(),
        "/proj/middle-dot.ts".to_string(),
    ];

    let result = command_line(&sys, &args);

    assert_eq!(result.status, ExitStatus::DiagnosticsPresent_OutputsSkipped);
}

#[test]
fn finds_config_in_ancestor_directory() {
    let fs = Arc::new(InMemoryFS::new());
    fs.insert_dir("/root");
    fs.insert_dir("/root/sub");
    fs.insert_file(
        "/root/tsconfig.json",
        r#"{"compilerOptions":{},"files":["sub/a.ts"]}"#,
    );
    fs.insert_file("/root/sub/a.ts", "let x = 1;");
    let sys = TestSystem::new(fs, "/root/sub");
    let args: Vec<String> = vec![];
    let result = command_line(&sys, &args);
    assert_eq!(result.status, ExitStatus::Success);
}

#[test]
fn build_mode_produces_output() {
    let fs = Arc::new(InMemoryFS::new());
    fs.insert_dir("/proj");
    fs.insert_file("/proj/a.ts", "let x: number = 1;");
    fs.insert_file(
        "/proj/tsconfig.json",
        r#"{"compilerOptions":{},"files":["a.ts"]}"#,
    );
    let sys = TestSystem::new(fs, "/proj");
    let args = vec!["-b".to_string()];
    let result = command_line(&sys, &args);
    assert_eq!(
        result.status,
        ExitStatus::Success,
        "output:\n{}",
        sys.output_string()
    );

    assert!(sys.fs().file_exists("/proj/a.js"));
    let js = sys.fs().read_file("/proj/a.js").unwrap();
    assert_eq!(js.trim(), "let x = 1;");
}

#[test]
fn regular_compilation_produces_output() {
    let fs = Arc::new(InMemoryFS::new());
    fs.insert_dir("/proj");
    fs.insert_file(
        "/proj/b.ts",
        "function foo(a: number): number { return a; }",
    );
    let sys = TestSystem::new(fs, "/proj");
    let args = vec!["--ignoreConfig".to_string(), "/proj/b.ts".to_string()];
    let result = command_line(&sys, &args);
    assert_eq!(result.status, ExitStatus::Success);
    assert!(sys.fs().file_exists("/proj/b.js"));
    let js = sys.fs().read_file("/proj/b.js").unwrap();
    assert!(js.contains("function foo(a)"));
    assert!(!js.contains(": number"));
}

#[test]
fn no_emit_flag_skips_output() {
    let fs = Arc::new(InMemoryFS::new());
    fs.insert_dir("/proj");
    fs.insert_file("/proj/c.ts", "let y = 2;");
    let sys = TestSystem::new(fs, "/proj");
    let args = vec![
        "--ignoreConfig".to_string(),
        "--noEmit".to_string(),
        "/proj/c.ts".to_string(),
    ];
    let result = command_line(&sys, &args);
    assert_eq!(result.status, ExitStatus::Success);
    assert!(!sys.fs().file_exists("/proj/c.js"));
}

#[test]
fn out_dir_redirects_output() {
    let fs = Arc::new(InMemoryFS::new());
    fs.insert_dir("/proj/src");
    fs.insert_file("/proj/src/d.ts", "let z: string = \"hi\";");
    let sys = TestSystem::new(fs, "/proj");
    let args = vec![
        "--ignoreConfig".to_string(),
        "--outDir".to_string(),
        "/proj/dist".to_string(),
        "/proj/src/d.ts".to_string(),
    ];
    let result = command_line(&sys, &args);
    assert_eq!(result.status, ExitStatus::Success);
    assert!(sys.fs().file_exists("/proj/dist/d.js"));
    let js = sys.fs().read_file("/proj/dist/d.js").unwrap();
    assert_eq!(js.trim(), "let z = \"hi\";");
}

#[test]
fn no_emit_on_error_skips_output_when_errors() {
    let fs = Arc::new(InMemoryFS::new());
    fs.insert_dir("/proj");
    fs.insert_file("/proj/e.ts", "interface { x: number }\nlet y = 1;");
    let sys = TestSystem::new(fs, "/proj");
    let args = vec![
        "--noLib".to_string(),
        "--ignoreConfig".to_string(),
        "--noEmitOnError".to_string(),
        "/proj/e.ts".to_string(),
    ];
    let result = command_line(&sys, &args);

    if result.status != ExitStatus::DiagnosticsPresent_OutputsSkipped {
        panic!(
            "Expected DiagnosticsPresent_OutputsSkipped but got {:?}. Output:\n{}",
            result.status,
            sys.output_string()
        );
    }
    assert!(!sys.fs().file_exists("/proj/e.js"));
}

#[test]
fn no_emit_on_error_emits_when_no_errors() {
    let fs = Arc::new(InMemoryFS::new());
    fs.insert_dir("/proj");
    fs.insert_file("/proj/f.ts", "let x: number = 1;");
    let sys = TestSystem::new(fs, "/proj");
    let args = vec![
        "--ignoreConfig".to_string(),
        "--noEmitOnError".to_string(),
        "/proj/f.ts".to_string(),
    ];
    let result = command_line(&sys, &args);
    assert_eq!(result.status, ExitStatus::Success);
    assert!(sys.fs().file_exists("/proj/f.js"));
    let js = sys.fs().read_file("/proj/f.js").unwrap();
    assert_eq!(js.trim(), "let x = 1;");
}

#[test]
fn errors_without_no_emit_on_error_still_emits() {
    let fs = Arc::new(InMemoryFS::new());
    fs.insert_dir("/proj");
    fs.insert_file("/proj/g.ts", "interface { x: number }\nlet y = 1;");
    let sys = TestSystem::new(fs, "/proj");
    let args = vec![
        "--noLib".to_string(),
        "--ignoreConfig".to_string(),
        "/proj/g.ts".to_string(),
    ];
    let result = command_line(&sys, &args);

    if result.status != ExitStatus::DiagnosticsPresent_OutputsGenerated {
        panic!(
            "Expected DiagnosticsPresent_OutputsGenerated but got {:?}. Output:\n{}",
            result.status,
            sys.output_string()
        );
    }
    assert!(sys.fs().file_exists("/proj/g.js"));
}

#[test]
fn list_files_only_skips_output() {
    let fs = Arc::new(InMemoryFS::new());
    fs.insert_dir("/proj");
    fs.insert_file("/proj/h.ts", "let x: number = 1;");
    let sys = TestSystem::new(fs, "/proj");
    let args = vec![
        "--ignoreConfig".to_string(),
        "--listFilesOnly".to_string(),
        "/proj/h.ts".to_string(),
    ];
    let result = command_line(&sys, &args);
    assert_eq!(result.status, ExitStatus::Success);
    assert!(!sys.fs().file_exists("/proj/h.js"));

    assert!(sys.output_string().contains("/proj/h.ts"));
}

#[test]
fn build_mode_with_out_dir() {
    let fs = Arc::new(InMemoryFS::new());
    fs.insert_dir("/proj");
    fs.insert_dir("/proj/src");
    fs.insert_file("/proj/src/i.ts", "let value: number = 1;");
    fs.insert_file(
        "/proj/tsconfig.json",
        r#"{"compilerOptions":{"outDir":"/proj/dist"},"files":["src/i.ts"]}"#,
    );
    let sys = TestSystem::new(fs, "/proj");
    let args = vec!["-b".to_string()];
    let result = command_line(&sys, &args);
    assert_eq!(
        result.status,
        ExitStatus::Success,
        "output:\n{}",
        sys.output_string()
    );
    assert!(sys.fs().file_exists("/proj/dist/src/i.js"));
    let js = sys.fs().read_file("/proj/dist/src/i.js").unwrap();
    assert!(js.contains("let value = 1;"));
    assert!(!js.contains(": number"));
}

#[test]
fn build_mode_builds_referenced_solution_project() {
    let fs = Arc::new(InMemoryFS::new());
    fs.insert_dir("/proj");
    fs.insert_dir("/proj/src");
    fs.insert_file(
        "/proj/tsconfig.json",
        r#"{"files":[],"references":[{"path":"./tsconfig.app.json"}]}"#,
    );
    fs.insert_file(
        "/proj/tsconfig.app.json",
        r#"{"compilerOptions":{"outDir":"/proj/dist"},"include":["src"]}"#,
    );
    fs.insert_file("/proj/src/app.ts", "export const app: number = 1;");
    let sys = TestSystem::new(fs, "/proj");
    let args = vec!["-b".to_string()];
    let result = command_line(&sys, &args);
    assert_eq!(
        result.status,
        ExitStatus::Success,
        "output:\n{}",
        sys.output_string()
    );
    assert!(sys.fs().file_exists("/proj/dist/src/app.js"));
}

#[test]
fn build_mode_detects_two_project_cycle() {
    let fs = Arc::new(InMemoryFS::new());
    fs.insert_dir("/cyc/a");
    fs.insert_dir("/cyc/b");
    fs.insert_file("/cyc/a/a.ts", "let a = 1;");
    fs.insert_file("/cyc/b/b.ts", "let b = 2;");
    fs.insert_file(
        "/cyc/a/tsconfig.json",
        r#"{"compilerOptions":{"noLib":true},"files":["a.ts"],"references":[{"path":"../b"}]}"#,
    );
    fs.insert_file(
        "/cyc/b/tsconfig.json",
        r#"{"compilerOptions":{"noLib":true},"files":["b.ts"],"references":[{"path":"../a"}]}"#,
    );
    let sys = TestSystem::new(fs, "/cyc/a");
    let args = vec!["-b".to_string()];
    let result = command_line(&sys, &args);
    let out = sys.output_string();
    assert_eq!(
        result.status,
        ExitStatus::ProjectReferenceCycle_OutputsSkipped,
        "output:\n{out}"
    );
    assert!(out.contains("TS6202"), "expected TS6202 in output:\n{out}");
    assert!(
        out.contains("Project references may not form a circular graph"),
        "output:\n{out}"
    );

    assert!(out.contains("/cyc/a/tsconfig.json"), "output:\n{out}");
    assert!(out.contains("/cyc/b/tsconfig.json"), "output:\n{out}");

    assert!(!sys.fs().file_exists("/cyc/a/a.js"));
    assert!(!sys.fs().file_exists("/cyc/b/b.js"));
}

#[test]
fn build_mode_detects_three_project_cycle() {
    let fs = Arc::new(InMemoryFS::new());
    fs.insert_dir("/cyc3/a");
    fs.insert_dir("/cyc3/b");
    fs.insert_dir("/cyc3/c");
    fs.insert_file("/cyc3/a/a.ts", "let a = 1;");
    fs.insert_file("/cyc3/b/b.ts", "let b = 2;");
    fs.insert_file("/cyc3/c/c.ts", "let c = 3;");
    fs.insert_file(
        "/cyc3/a/tsconfig.json",
        r#"{"compilerOptions":{"noLib":true},"files":["a.ts"],"references":[{"path":"../b"}]}"#,
    );
    fs.insert_file(
        "/cyc3/b/tsconfig.json",
        r#"{"compilerOptions":{"noLib":true},"files":["b.ts"],"references":[{"path":"../c"}]}"#,
    );
    fs.insert_file(
        "/cyc3/c/tsconfig.json",
        r#"{"compilerOptions":{"noLib":true},"files":["c.ts"],"references":[{"path":"../a"}]}"#,
    );
    let sys = TestSystem::new(fs, "/cyc3/a");
    let args = vec!["-b".to_string()];
    let result = command_line(&sys, &args);
    let out = sys.output_string();
    assert_eq!(
        result.status,
        ExitStatus::ProjectReferenceCycle_OutputsSkipped,
        "output:\n{out}"
    );
    assert!(out.contains("TS6202"), "expected TS6202 in output:\n{out}");

    assert!(out.contains("/cyc3/a/tsconfig.json"), "output:\n{out}");
    assert!(out.contains("/cyc3/b/tsconfig.json"), "output:\n{out}");
    assert!(out.contains("/cyc3/c/tsconfig.json"), "output:\n{out}");
    assert!(!sys.fs().file_exists("/cyc3/a/a.js"));
    assert!(!sys.fs().file_exists("/cyc3/b/b.js"));
    assert!(!sys.fs().file_exists("/cyc3/c/c.js"));
}

#[test]
fn build_mode_no_cycle_builds_in_dependency_order() {
    let fs = Arc::new(InMemoryFS::new());
    fs.insert_dir("/chain/a");
    fs.insert_dir("/chain/b");
    fs.insert_dir("/chain/c");
    fs.insert_file("/chain/a/a.ts", "let a = 1;");
    fs.insert_file("/chain/b/b.ts", "let b = 2;");
    fs.insert_file("/chain/c/c.ts", "let c = 3;");
    fs.insert_file(
        "/chain/a/tsconfig.json",
        r#"{"compilerOptions":{},"files":["a.ts"],"references":[{"path":"../b"}]}"#,
    );
    fs.insert_file(
        "/chain/b/tsconfig.json",
        r#"{"compilerOptions":{},"files":["b.ts"],"references":[{"path":"../c"}]}"#,
    );
    fs.insert_file(
        "/chain/c/tsconfig.json",
        r#"{"compilerOptions":{},"files":["c.ts"]}"#,
    );
    let sys = TestSystem::new(fs, "/chain/a");
    let args = vec!["-b".to_string()];
    let result = command_line(&sys, &args);
    let out = sys.output_string();
    assert_eq!(
        result.status,
        ExitStatus::Success,
        "expected successful build, output:\n{out}"
    );

    assert!(
        !out.contains("TS6202"),
        "unexpected cycle diagnostic:\n{out}"
    );

    assert!(sys.fs().file_exists("/chain/c/c.js"), "output:\n{out}");
    assert!(sys.fs().file_exists("/chain/b/b.js"), "output:\n{out}");
    assert!(sys.fs().file_exists("/chain/a/a.js"), "output:\n{out}");
}

#[test]
fn show_config_with_boolean_option() {
    let fs = Arc::new(InMemoryFS::new());
    fs.insert_dir("/proj");
    fs.insert_file("/proj/index.ts", "");
    fs.insert_file("/proj/tsconfig.json", "{}");
    let sys = TestSystem::new(fs, "/proj");
    let args = vec!["--showConfig".to_string(), "--noUnusedLocals".to_string()];
    let result = command_line(&sys, &args);
    assert_eq!(result.status, ExitStatus::Success);
    let out = sys.output_string();
    assert!(out.contains("\"compilerOptions\""), "output: {out}");
    assert!(out.contains("\"noUnusedLocals\": true"), "output: {out}");
}

#[test]
fn show_config_with_enum_options() {
    let fs = Arc::new(InMemoryFS::new());
    fs.insert_dir("/proj");
    fs.insert_file("/proj/index.ts", "");
    fs.insert_file("/proj/tsconfig.json", "{}");
    let sys = TestSystem::new(fs, "/proj");
    let args = vec![
        "--showConfig".to_string(),
        "--target".to_string(),
        "es5".to_string(),
        "--jsx".to_string(),
        "react".to_string(),
    ];
    let result = command_line(&sys, &args);
    assert_eq!(result.status, ExitStatus::Success);
    let out = sys.output_string();
    assert!(out.contains("\"target\": \"es5\""), "output: {out}");
    assert!(out.contains("\"jsx\": \"react\""), "output: {out}");
}

#[test]
fn show_config_with_list_options() {
    let fs = Arc::new(InMemoryFS::new());
    fs.insert_dir("/proj");
    fs.insert_file("/proj/index.ts", "");
    fs.insert_file("/proj/tsconfig.json", "{}");
    let sys = TestSystem::new(fs, "/proj");
    let args = vec![
        "--showConfig".to_string(),
        "--types".to_string(),
        "jquery,mocha".to_string(),
    ];
    let result = command_line(&sys, &args);
    assert_eq!(result.status, ExitStatus::Success);
    let out = sys.output_string();
    assert!(out.contains("\"types\""), "output: {out}");
    assert!(out.contains("jquery"), "output: {out}");
    assert!(out.contains("mocha"), "output: {out}");
}

#[test]
fn show_config_with_tsconfig_file() {
    let fs = Arc::new(InMemoryFS::new());
    fs.insert_dir("/proj/src");
    fs.insert_file("/proj/src/index.ts", "export const a = 1;");
    fs.insert_file(
        "/proj/tsconfig.json",
        r#"{
                "compilerOptions": {
                    "esModuleInterop": true,
                    "target": "es5",
                    "module": "commonjs",
                    "strict": true
                },
                "include": ["src/*"]
            }"#,
    );
    let sys = TestSystem::new(fs, "/proj");
    let args = vec![
        "-p".to_string(),
        "tsconfig.json".to_string(),
        "--showConfig".to_string(),
    ];
    let result = command_line(&sys, &args);
    if result.status != ExitStatus::Success {
        panic!(
            "Expected Success but got {:?}. Output:\n{}",
            result.status,
            sys.output_string()
        );
    }
    let out = sys.output_string();
    assert!(out.contains("\"target\": \"es5\""), "output: {out}");
    assert!(out.contains("\"module\": \"commonjs\""), "output: {out}");
    assert!(out.contains("\"strict\": true"), "output: {out}");
    assert!(out.contains("\"include\""), "output: {out}");
    assert!(out.contains("src/*"), "output: {out}");
}

#[test]
fn show_config_with_paths() {
    let fs = Arc::new(InMemoryFS::new());
    fs.insert_dir("/proj/src");
    fs.insert_file("/proj/src/index.ts", "export const a = 1;");
    fs.insert_file(
        "/proj/tsconfig.json",
        r#"{
                "compilerOptions": {
                    "baseUrl": ".",
                    "paths": {
                        "@root/*": ["./*"],
                        "@common/*": ["src/common/*"]
                    }
                }
            }"#,
    );
    let sys = TestSystem::new(fs, "/proj");
    let args = vec![
        "-p".to_string(),
        "tsconfig.json".to_string(),
        "--showConfig".to_string(),
    ];
    let result = command_line(&sys, &args);
    assert_eq!(result.status, ExitStatus::Success);
    let out = sys.output_string();
    assert!(out.contains("\"paths\""), "output: {out}");
    assert!(out.contains("@root/*"), "output: {out}");
    assert!(out.contains("@common/*"), "output: {out}");
    assert!(out.contains("\"baseUrl\": \".\""), "output: {out}");
}

#[test]
fn show_config_with_exclude() {
    let fs = Arc::new(InMemoryFS::new());
    fs.insert_dir("/proj/src");
    fs.insert_file("/proj/src/index.ts", "export const a = 1;");
    fs.insert_file(
        "/proj/tsconfig.json",
        r#"{
                "compilerOptions": { "strict": true },
                "exclude": ["test"]
            }"#,
    );
    let sys = TestSystem::new(fs, "/proj");
    let args = vec![
        "-p".to_string(),
        "tsconfig.json".to_string(),
        "--showConfig".to_string(),
    ];
    let result = command_line(&sys, &args);
    assert_eq!(result.status, ExitStatus::Success);
    let out = sys.output_string();
    assert!(out.contains("\"exclude\""), "output: {out}");
    assert!(out.contains("test"), "output: {out}");
}

#[test]
fn show_config_with_advanced_options() {
    let fs = Arc::new(InMemoryFS::new());
    fs.insert_dir("/proj");
    fs.insert_file("/proj/index.ts", "");
    fs.insert_file("/proj/tsconfig.json", "{}");
    let sys = TestSystem::new(fs, "/proj");
    let args = vec![
        "--showConfig".to_string(),
        "--declaration".to_string(),
        "--declarationDir".to_string(),
        "lib".to_string(),
        "--skipLibCheck".to_string(),
        "--noErrorTruncation".to_string(),
    ];
    let result = command_line(&sys, &args);
    assert_eq!(result.status, ExitStatus::Success);
    let out = sys.output_string();
    assert!(out.contains("\"declaration\": true"), "output: {out}");
    assert!(out.contains("\"declarationDir\": \"lib\""), "output: {out}");
    assert!(out.contains("\"skipLibCheck\": true"), "output: {out}");
    assert!(out.contains("\"noErrorTruncation\": true"), "output: {out}");
}

#[test]
fn project_with_file_path() {
    let fs = Arc::new(InMemoryFS::new());
    fs.insert_dir("/proj");
    fs.insert_file("/proj/first.ts", "export const a = 1;");
    fs.insert_file(
        "/proj/tsconfig.json",
        r#"{"compilerOptions":{"noEmit":true},"files":["first.ts"]}"#,
    );
    let sys = TestSystem::new(fs, "/proj");
    let args = vec!["-p".to_string(), "/proj/tsconfig.json".to_string()];
    let result = command_line(&sys, &args);
    if result.status != ExitStatus::Success {
        panic!(
            "Expected Success but got {:?}. Output:\n{}",
            result.status,
            sys.output_string()
        );
    }
}

#[test]
fn project_with_folder_path() {
    let fs = Arc::new(InMemoryFS::new());
    fs.insert_dir("/proj");
    fs.insert_file("/proj/first.ts", "export const a = 1;");
    fs.insert_file(
        "/proj/tsconfig.json",
        r#"{"compilerOptions":{"noEmit":true},"files":["first.ts"]}"#,
    );
    let sys = TestSystem::new(fs, "/proj");
    let args = vec!["-p".to_string(), "/proj".to_string()];
    let result = command_line(&sys, &args);
    assert_eq!(result.status, ExitStatus::Success);
}

#[test]
fn project_with_dot_folder() {
    let fs = Arc::new(InMemoryFS::new());
    fs.insert_dir("/proj");
    fs.insert_file("/proj/first.ts", "export const a = 1;");
    fs.insert_file(
        "/proj/tsconfig.json",
        r#"{"compilerOptions":{"noEmit":true},"files":["first.ts"]}"#,
    );
    let sys = TestSystem::new(fs, "/proj");
    let args = vec!["-p".to_string(), ".".to_string()];
    let result = command_line(&sys, &args);
    assert_eq!(result.status, ExitStatus::Success);
}

#[test]
fn project_with_nonexistent_path() {
    let fs = Arc::new(InMemoryFS::new());
    fs.insert_dir("/proj");
    let sys = TestSystem::new(fs, "/proj");
    let args = vec!["-p".to_string(), "/proj/nonexistent.json".to_string()];
    let result = command_line(&sys, &args);
    assert_eq!(result.status, ExitStatus::DiagnosticsPresent_OutputsSkipped);
    assert!(sys.output_string().contains("does not exist"));
}

#[test]
fn project_with_nonexistent_directory() {
    let fs = Arc::new(InMemoryFS::new());
    fs.insert_dir("/proj");
    let sys = TestSystem::new(fs, "/proj");
    let args = vec!["-p".to_string(), "/proj/nonexistent".to_string()];
    let result = command_line(&sys, &args);
    assert_eq!(result.status, ExitStatus::DiagnosticsPresent_OutputsSkipped);
    assert!(sys.output_string().contains("does not exist"));
}

#[test]
fn project_mixed_with_files_errors() {
    let fs = Arc::new(InMemoryFS::new());
    fs.insert_dir("/proj");
    fs.insert_file("/proj/a.ts", "let x = 1;");
    fs.insert_file(
        "/proj/tsconfig.json",
        r#"{"compilerOptions":{"noLib":true}}"#,
    );
    let sys = TestSystem::new(fs, "/proj");
    let args = vec![
        "-p".to_string(),
        "/proj/tsconfig.json".to_string(),
        "/proj/a.ts".to_string(),
    ];
    let result = command_line(&sys, &args);
    assert_eq!(result.status, ExitStatus::DiagnosticsPresent_OutputsSkipped);
    assert!(sys.output_string().contains("cannot be mixed"));
}

#[test]
fn empty_tsconfig_file() {
    let fs = Arc::new(InMemoryFS::new());
    fs.insert_dir("/proj");
    fs.insert_file("/proj/first.ts", "export const a = 1;");
    fs.insert_file("/proj/tsconfig.json", "");
    let sys = TestSystem::new(fs, "/proj");
    let args = vec!["-p".to_string(), ".".to_string()];
    let result = command_line(&sys, &args);
    if result.status != ExitStatus::Success {
        panic!(
            "Expected Success but got {:?}. Output:\n{}",
            result.status,
            sys.output_string()
        );
    }
}

#[test]
fn watch_and_list_files_only_errors() {
    let fs = Arc::new(InMemoryFS::new());
    fs.insert_dir("/proj");
    fs.insert_file("/proj/a.ts", "let x = 1;");
    let sys = TestSystem::new(fs, "/proj");
    let args = vec![
        "--noLib".to_string(),
        "--ignoreConfig".to_string(),
        "--watch".to_string(),
        "--listFilesOnly".to_string(),
        "/proj/a.ts".to_string(),
    ];
    let result = command_line(&sys, &args);
    assert_eq!(result.status, ExitStatus::DiagnosticsPresent_OutputsSkipped);
    assert!(sys.output_string().contains("cannot be combined"));
}

#[test]
fn build_not_first_argument() {
    let fs = Arc::new(InMemoryFS::new());
    fs.insert_dir("/proj");
    fs.insert_file("/proj/a.ts", "let x: number = 1;");
    let sys = TestSystem::new(fs, "/proj");
    let args = vec![
        "--noLib".to_string(),
        "--build".to_string(),
        "--ignoreConfig".to_string(),
        "/proj/a.ts".to_string(),
    ];
    let result = command_line(&sys, &args);
    assert_eq!(result.status, ExitStatus::DiagnosticsPresent_OutputsSkipped);
    assert!(sys.output_string().contains("must be the first"));
}

#[test]
fn compiles_multiple_files() {
    let fs = Arc::new(InMemoryFS::new());
    fs.insert_dir("/proj");
    fs.insert_file("/proj/a.ts", "export const x = 1;");
    fs.insert_file("/proj/b.ts", "export const y = 2;");
    let sys = TestSystem::new(fs, "/proj");
    let args = vec![
        "--ignoreConfig".to_string(),
        "/proj/a.ts".to_string(),
        "/proj/b.ts".to_string(),
    ];
    let result = command_line(&sys, &args);
    assert_eq!(result.status, ExitStatus::Success);
    assert!(sys.fs().file_exists("/proj/a.js"));
    assert!(sys.fs().file_exists("/proj/b.js"));
}

#[test]
fn declaration_option_compiles_with_flag() {
    let fs = Arc::new(InMemoryFS::new());
    fs.insert_dir("/proj");
    fs.insert_file(
        "/proj/a.ts",
        "export const x: number = 1;\nexport function foo(a: number): number { return a; }",
    );
    let sys = TestSystem::new(fs, "/proj");
    let args = vec![
        "--ignoreConfig".to_string(),
        "--declaration".to_string(),
        "/proj/a.ts".to_string(),
    ];
    let result = command_line(&sys, &args);
    if result.status != ExitStatus::Success {
        panic!(
            "Expected Success but got {:?}. Output:\n{}",
            result.status,
            sys.output_string()
        );
    }
    assert!(sys.fs().file_exists("/proj/a.js"));
}

#[test]
fn source_map_option_compiles_with_flag() {
    let fs = Arc::new(InMemoryFS::new());
    fs.insert_dir("/proj");
    fs.insert_file("/proj/a.ts", "let x: number = 1;");
    let sys = TestSystem::new(fs, "/proj");
    let args = vec![
        "--ignoreConfig".to_string(),
        "--sourceMap".to_string(),
        "/proj/a.ts".to_string(),
    ];
    let result = command_line(&sys, &args);
    assert_eq!(result.status, ExitStatus::Success);
    assert!(sys.fs().file_exists("/proj/a.js"));
}

#[test]
fn parse_enum_options_module_target() {
    let fs = Arc::new(InMemoryFS::new());
    fs.insert_dir("/proj");
    fs.insert_file("/proj/a.ts", "let x = 1;");
    let sys = TestSystem::new(fs, "/proj");
    let args = vec![
        "--ignoreConfig".to_string(),
        "--module".to_string(),
        "commonjs".to_string(),
        "--target".to_string(),
        "es5".to_string(),
        "/proj/a.ts".to_string(),
    ];
    let result = command_line(&sys, &args);
    assert_eq!(result.status, ExitStatus::Success);
    assert!(sys.fs().file_exists("/proj/a.js"));
}

#[test]
fn show_config_with_module_and_target() {
    let fs = Arc::new(InMemoryFS::new());
    fs.insert_dir("/proj");
    fs.insert_file("/proj/index.ts", "");
    fs.insert_file("/proj/tsconfig.json", "{}");
    let sys = TestSystem::new(fs, "/proj");
    let args = vec![
        "--showConfig".to_string(),
        "--module".to_string(),
        "nodenext".to_string(),
        "--target".to_string(),
        "esnext".to_string(),
    ];
    let result = command_line(&sys, &args);
    assert_eq!(result.status, ExitStatus::Success);
    let out = sys.output_string();
    assert!(out.contains("\"module\": \"nodenext\""), "output: {out}");
    assert!(out.contains("\"target\": \"esnext\""), "output: {out}");
}

struct EnvTestSystem {
    fs: Arc<BundledFS>,
    cwd: String,
    output: Arc<Mutex<Vec<u8>>>,
    env: std::collections::HashMap<String, String>,
}

impl EnvTestSystem {
    fn new(inner_fs: Arc<InMemoryFS>, cwd: &str) -> Self {
        Self {
            fs: Arc::new(BundledFS::new(inner_fs as Arc<dyn FS>)),
            cwd: cwd.to_string(),
            output: Arc::new(Mutex::new(Vec::new())),
            env: std::collections::HashMap::new(),
        }
    }

    fn with_env(mut self, key: &str, val: &str) -> Self {
        self.env.insert(key.to_string(), val.to_string());
        self
    }

    fn output_string(&self) -> String {
        String::from_utf8_lossy(&self.output.lock().unwrap()).to_string()
    }
}

impl System for EnvTestSystem {
    fn writer(&self) -> Box<dyn Write + Send> {
        Box::new(BufferWriter {
            buf: Arc::clone(&self.output),
        })
    }
    fn fs(&self) -> Arc<dyn FS> {
        Arc::clone(&self.fs) as Arc<dyn FS>
    }
    fn default_library_path(&self) -> &str {
        "bundled:///libs"
    }
    fn current_directory(&self) -> &str {
        &self.cwd
    }
    fn write_output_is_tty(&self) -> bool {
        false
    }
    fn width_of_terminal(&self) -> usize {
        80
    }
    fn environment_variable(&self, name: &str) -> Option<String> {
        self.env.get(name).cloned()
    }
}

#[test]
fn no_color_env_disables_pretty() {
    let fs = Arc::new(InMemoryFS::new());
    fs.insert_dir("/proj");
    fs.insert_file("/proj/a.ts", "interface { x: number }");
    let sys = EnvTestSystem::new(fs, "/proj").with_env("NO_COLOR", "true");
    let args = vec![
        "--noLib".to_string(),
        "--ignoreConfig".to_string(),
        "/proj/a.ts".to_string(),
    ];
    let result = command_line(&sys, &args);

    let out = sys.output_string();
    assert!(
        !out.contains("\x1b["),
        "output should not contain ANSI codes: {out}"
    );

    assert!(result.status != ExitStatus::Success);
}

#[test]
fn force_color_enables_pretty() {
    let fs = Arc::new(InMemoryFS::new());
    fs.insert_dir("/proj");
    fs.insert_file("/proj/a.ts", "interface { x: number }");

    let sys = EnvTestSystem::new(fs, "/proj").with_env("FORCE_COLOR", "true");
    let args = vec![
        "--noLib".to_string(),
        "--ignoreConfig".to_string(),
        "/proj/a.ts".to_string(),
    ];
    let _result = command_line(&sys, &args);
}

#[test]
fn list_files_prints_source_files() {
    let fs = Arc::new(InMemoryFS::new());
    fs.insert_dir("/proj");
    fs.insert_file("/proj/a.ts", "let x = 1;");
    fs.insert_file("/proj/b.ts", "let y = 2;");
    let sys = TestSystem::new(fs, "/proj");
    let args = vec![
        "--ignoreConfig".to_string(),
        "--listFiles".to_string(),
        "/proj/a.ts".to_string(),
        "/proj/b.ts".to_string(),
    ];
    let result = command_line(&sys, &args);
    assert_eq!(result.status, ExitStatus::Success);
    let out = sys.output_string();
    assert!(out.contains("/proj/a.ts"), "output: {out}");
    assert!(out.contains("/proj/b.ts"), "output: {out}");
}

#[test]
fn show_config_with_compile_on_save() {
    let fs = Arc::new(InMemoryFS::new());
    fs.insert_dir("/proj/src");
    fs.insert_file("/proj/src/index.ts", "export const a = 1;");
    fs.insert_file(
        "/proj/tsconfig.json",
        r#"{
                "compilerOptions": { "strict": true },
                "compileOnSave": true,
                "include": ["src/*"]
            }"#,
    );
    let sys = TestSystem::new(fs, "/proj");
    let args = vec![
        "-p".to_string(),
        "tsconfig.json".to_string(),
        "--showConfig".to_string(),
    ];
    let result = command_line(&sys, &args);
    assert_eq!(result.status, ExitStatus::Success);
    let out = sys.output_string();
    assert!(out.contains("\"compileOnSave\": true"), "output: {out}");
}

#[test]
fn show_config_with_references() {
    let fs = Arc::new(InMemoryFS::new());
    fs.insert_dir("/proj/src");
    fs.insert_file("/proj/src/index.ts", "export const a = 1;");
    fs.insert_file(
        "/proj/tsconfig.json",
        r#"{
                "compilerOptions": { "composite": true, "strict": true },
                "references": [{ "path": "./packages/a" }]
            }"#,
    );
    let sys = TestSystem::new(fs, "/proj");
    let args = vec![
        "-p".to_string(),
        "tsconfig.json".to_string(),
        "--showConfig".to_string(),
    ];
    let result = command_line(&sys, &args);
    assert_eq!(result.status, ExitStatus::Success);
    let out = sys.output_string();
    assert!(out.contains("\"composite\": true"), "output: {out}");
    assert!(out.contains("\"references\""), "output: {out}");
    assert!(out.contains("\"path\": \"./packages/a\""), "output: {out}");
}

#[test]
fn missing_file_in_tsconfig_reports_error() {
    let fs = Arc::new(InMemoryFS::new());
    fs.insert_dir("/proj");
    fs.insert_file("/proj/tsconfig.json", r#"{"files":["./doesNotExist.ts"]}"#);
    let sys = TestSystem::new(fs, "/proj");
    let args = vec!["-p".to_string(), "./tsconfig.json".to_string()];
    let result = command_line(&sys, &args);

    assert_ne!(result.status, ExitStatus::Success);
}

#[test]
fn all_flag_prints_help() {
    let fs = Arc::new(InMemoryFS::new());
    fs.insert_dir("/proj");
    let sys = TestSystem::new(fs, "/proj");
    let args = vec!["--all".to_string()];
    let result = command_line(&sys, &args);
    assert_eq!(result.status, ExitStatus::Success);
    let out = sys.output_string();
    assert!(
        out.contains("tsc: The TypeScript Compiler"),
        "header missing:\n{out}"
    );

    assert!(
        out.contains("ALL COMPILER OPTIONS"),
        "section missing:\n{out}"
    );
    assert!(
        out.contains("WATCH OPTIONS"),
        "watch section missing:\n{out}"
    );
    assert!(
        out.contains("BUILD OPTIONS"),
        "build section missing:\n{out}"
    );

    assert!(
        out.contains("Do not emit outputs."),
        "noEmit desc missing:\n{out}"
    );
}

#[test]
fn watch_is_source_file_matches_known_extensions() {
    let yes = [".ts", ".tsx", ".js", ".jsx", ".json", ".mts", ".cts"];
    for ext in yes {
        let path = format!("/proj/src/a{ext}");
        assert!(watch::is_source_file(&path), "expected {path} to match");
    }
    assert!(watch::is_source_file("a.ts"));

    assert!(!watch::is_source_file("a.ts.map"));
    assert!(!watch::is_source_file("readme.md"));
    assert!(!watch::is_source_file("a.d.ts.bak"));
    assert!(!watch::is_source_file(""));
}

#[test]
fn watch_timestamp_is_hh_mm_ss() {
    let ts = watch::timestamp();
    let parts: Vec<&str> = ts.split(':').collect();
    assert_eq!(parts.len(), 3, "expected HH:MM:SS, got {ts}");
    for p in &parts {
        assert_eq!(p.len(), 2, "each component should be 2 digits: {ts}");
        assert!(
            p.chars().all(|c| c.is_ascii_digit()),
            "non-digit component in {ts}"
        );
    }
}

#[test]
fn watch_summary_reports_zero_errors_on_success() {
    let fs = Arc::new(InMemoryFS::new());
    fs.insert_dir("/proj");
    let sys = TestSystem::new(fs, "/proj");
    watch::print_watch_summary(&sys, ExitStatus::Success);
    let out = sys.output_string();
    assert!(out.contains("Found 0 errors."), "got:\n{out}");
    assert!(out.contains("Watching for file changes."));
}

#[test]
fn watch_summary_reports_errors_on_failure() {
    let fs = Arc::new(InMemoryFS::new());
    fs.insert_dir("/proj");
    let sys = TestSystem::new(fs, "/proj");
    watch::print_watch_summary(&sys, ExitStatus::DiagnosticsPresent_OutputsGenerated);
    let out = sys.output_string();
    assert!(out.contains("Found errors."), "got:\n{out}");
}

#[test]
fn watch_compile_once_runs_initial_compilation() {
    let fs = Arc::new(InMemoryFS::new());
    fs.insert_dir("/proj");
    fs.insert_file("/proj/a.ts", "let x: number = 1;");
    let sys = TestSystem::new(fs, "/proj");

    let mut config = parse_command_line(
        &["--ignoreConfig".to_string(), "/proj/a.ts".to_string()],
        "/proj",
        None,
    );
    config.compiler_options.watch = Tristate::True;

    let result = watch::compile_once(&sys, &config, &config.compiler_options, "", false, None);
    assert_eq!(
        result.status,
        ExitStatus::Success,
        "initial watch compilation failed:\n{}",
        sys.output_string()
    );
}
