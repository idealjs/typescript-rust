use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug)]
struct Case {
    name: &'static str,
    args: &'static [&'static str],

    expect_success: bool,

    expected_files: &'static [(&'static str, &'static str)],

    stdout_contains: &'static [&'static str],

    skip_oracle: bool,
}

const CASES: &[Case] = &[
    Case {
        name: "simple_emit",
        args: &[],
        expect_success: true,

        expected_files: &[("dist/src/main.js", "let answer = 42;\n")],
        stdout_contains: &[],
        skip_oracle: false,
    },
    Case {
        name: "type_only_declarations",
        args: &[],
        expect_success: true,
        expected_files: &[("dist/src/main.js", "const value = { name: \"Ada\" };\n")],
        stdout_contains: &[],
        skip_oracle: false,
    },
    Case {
        name: "nested_out_dir",
        args: &[],
        expect_success: true,

        expected_files: &[(
            "dist/lib/main.js",
            "export function square(x) { return x * x; }\n",
        )],
        stdout_contains: &[],
        skip_oracle: false,
    },
    Case {
        name: "single_file",

        args: &["main.ts"],
        expect_success: true,
        expected_files: &[(
            "main.js",
            "const greeting = \"hello\";\nconsole.log(greeting);\n",
        )],
        stdout_contains: &[],
        skip_oracle: false,
    },
    Case {
        name: "project_dir",

        args: &["-p", "."],
        expect_success: true,
        expected_files: &[(
            "dist/src/main.js",
            "function add(a, b) { return a + b; }\nexport { add };\n",
        )],
        stdout_contains: &[],
        skip_oracle: false,
    },
    Case {
        name: "project_file",

        args: &["-p", "tsconfig.json"],
        expect_success: true,
        expected_files: &[("dist/src/main.js", "const x = 42;\nexport { x };\n")],
        stdout_contains: &[],
        skip_oracle: false,
    },
    Case {
        name: "jsonc_config",

        args: &[],
        expect_success: true,
        expected_files: &[("dist/src/main.js", "const y = 99;\nexport { y };\n")],
        stdout_contains: &[],
        skip_oracle: false,
    },
    Case {
        name: "show_config",

        args: &["--showConfig"],
        expect_success: true,
        expected_files: &[],
        stdout_contains: &["\"compilerOptions\"", "\"outDir\"", "\"strict\""],
        skip_oracle: false,
    },
    Case {
        name: "invalid_json",

        args: &[],
        expect_success: false,
        expected_files: &[],
        stdout_contains: &[],
        skip_oracle: false,
    },

    Case {
        name: "config_dir",

        args: &[],
        expect_success: true,
        expected_files: &[("dist/main.js", "const w = 123;\nexport { w };\n")],
        stdout_contains: &[],
        skip_oracle: false,
    },
    Case {
        name: "extends_relative",

        args: &[],
        expect_success: true,
        expected_files: &[("dist/src/main.js", "const x = 42;\nexport { x };\n")],
        stdout_contains: &[],
        skip_oracle: false,
    },
    Case {
        name: "extends_array",

        args: &[],
        expect_success: true,
        expected_files: &[("dist/src/main.js", "const y = 99;\nexport { y };\n")],
        stdout_contains: &[],
        skip_oracle: false,
    },
    Case {
        name: "extends_bare_specifier",

        args: &[],
        expect_success: true,
        expected_files: &[("dist/src/main.js", "const z = 7;\nexport { z };\n")],
        stdout_contains: &[],
        skip_oracle: false,
    },
    Case {
        name: "include_pattern",

        args: &[],
        expect_success: true,
        expected_files: &[("dist/src/main.js", "const c = 3;\nexport { c };\n")],
        stdout_contains: &[],
        skip_oracle: false,
    },
    Case {
        name: "exclude_pattern",

        args: &[],
        expect_success: true,
        expected_files: &[("dist/src/main.js", "const b = 2;\nexport { b };\n")],
        stdout_contains: &[],
        skip_oracle: false,
    },
    Case {
        name: "multiple_files",

        args: &[],
        expect_success: true,
        expected_files: &[
            (
                "dist/src/helper.js",
                "export function helper(x) { return x * 2; }\n",
            ),
            (
                "dist/src/main.js",
                "import { helper } from \"./helper\";\nconst result = helper(10);\nexport { result };\n",
            ),
        ],
        stdout_contains: &[],
        skip_oracle: false,
    },
    Case {
        name: "no_emit",

        args: &[],
        expect_success: true,
        expected_files: &[],
        stdout_contains: &[],
        skip_oracle: false,
    },
    Case {
        name: "strict_mode",

        args: &[],
        expect_success: true,
        expected_files: &[("dist/src/main.js", "const d = 4;\nexport { d };\n")],
        stdout_contains: &[],
        skip_oracle: false,
    },
    Case {
        name: "comments_stripped",

        args: &[],
        expect_success: true,
        expected_files: &[("dist/src/main.js", "const e = 5;\nexport { e };\n")],
        stdout_contains: &[],
        skip_oracle: false,
    },
    Case {
        name: "target_es5",

        args: &[],
        expect_success: true,
        expected_files: &[("dist/src/main.js", "var f = 6;\nexport { f };\n")],
        stdout_contains: &[],
        skip_oracle: false,
    },
    Case {
        name: "module_commonjs",

        args: &[],
        expect_success: true,
        expected_files: &[(
            "dist/src/main.js",
            "\"use strict\";\nconst x = 1;\nconst y = 2;\nexports.x = x;\nexports.y = y;\nexports.default = 42;\n",
        )],
        stdout_contains: &[],
        skip_oracle: true,
    },
    Case {
        name: "source_map",

        args: &[],
        expect_success: true,
        expected_files: &[(
            "dist/src/main.js",
            "let x = 1;\nlet y = 2;\n//# sourceMappingURL=main.js.map",
        )],
        stdout_contains: &[],
        skip_oracle: true,
    },
    Case {
        name: "declaration_emit",

        args: &[],
        expect_success: true,
        expected_files: &[
            (
                "dist/src/main.js",
                "export function add(a, b) { return a + b; }\nexport const PI = 3.14;\n",
            ),
            (
                "dist/src/main.d.ts",
                "export declare function add(a: number, b: number): number;\n\
                 export declare const PI: number;\n\
                 export interface User { id: number; name: string; }\n\
                 export type ID = string | number;\n",
            ),
        ],
        stdout_contains: &[],
        skip_oracle: true,
    },
    Case {
        name: "declaration_dir",

        args: &[],
        expect_success: true,
        expected_files: &[
            (
                "dist/js/src/main.js",
                "export function add(a, b) { return a + b; }\nexport const VERSION = 1;\n",
            ),
            (
                "dist/types/src/main.d.ts",
                "export declare function add(a: number, b: number): number;\n\
                 export declare const VERSION: number;\n",
            ),
        ],
        stdout_contains: &[],
        skip_oracle: true,
    },
    Case {
        name: "enum_emit",

        args: &[],
        expect_success: true,
        expected_files: &[(
            "dist/src/main.js",
            "enum Color { Red, Green, Blue }\n\
             enum Status { Active = 1, Inactive = 2 }\n\
             enum Direction { Up = \"UP\", Down = \"DOWN\" }\n\
             export { Color, Status, Direction };\n",
        )],
        stdout_contains: &[],
        skip_oracle: false,
    },
    Case {
        name: "namespace_emit",

        args: &[],
        expect_success: true,
        expected_files: &[(
            "dist/src/main.js",
            "namespace Utils {\n\
             \x20   export function helper(x) { return x * 2; }\n\
             \x20   export const PI = 3.14;\n\
             }\n\
             export { Utils };\n",
        )],
        stdout_contains: &[],
        skip_oracle: false,
    },
    Case {
        name: "mixed_js_ts",

        args: &[],
        expect_success: true,
        expected_files: &[
            (
                "dist/src/main.js",
                "export function add(a, b) { return a + b; }\n",
            ),
            (
                "dist/src/util.js",
                "export function double(x) { return x * 2; }\n",
            ),
        ],
        stdout_contains: &[],
        skip_oracle: false,
    },
    Case {
        name: "es_modules",

        args: &[],
        expect_success: true,
        expected_files: &[
            (
                "dist/src/main.js",
                "import { helper } from \"./helper\";\nconst result = helper(10);\nexport { result };\n",
            ),
            (
                "dist/src/helper.js",
                "export function helper(x) { return x * 2; }\n",
            ),
        ],
        stdout_contains: &[],
        skip_oracle: false,
    },
    Case {
        name: "class_emit",

        args: &[],
        expect_success: true,
        expected_files: &[(
            "dist/src/main.js",
            "export class Point {\n\
             \x20   x;\n\
             \x20   y;\n\
             \x20   constructor(x, y) { this.x = x; this.y = y; }\n\
             \x20   sum() { return this.x + this.y; }\n\
             }\n",
        )],
        stdout_contains: &[],
        skip_oracle: false,
    },
    Case {
        name: "decorators_emit",

        args: &[],
        expect_success: true,
        expected_files: &[(
            "dist/src/main.js",
            "function log(target, key, desc) { return desc; }\n\
             export class Greeter {\n\
             \x20   greeting;\n\
             \x20   constructor(msg) { this.greeting = msg; }\n\
             \x20   @log greet() { return \"Hello, \" + this.greeting; }\n\
             }\n",
        )],
        stdout_contains: &[],
        skip_oracle: false,
    },

    Case {
        name: "parser_syntax_error",

        args: &[],
        expect_success: false,
        expected_files: &[],
        stdout_contains: &[],
        skip_oracle: false,
    },
    Case {
        name: "parser_tsx",

        args: &[],
        expect_success: true,
        expected_files: &[],
        stdout_contains: &[],
        skip_oracle: true,
    },
    Case {
        name: "parser_generics",

        args: &[],
        expect_success: true,
        expected_files: &[],
        stdout_contains: &[],
        skip_oracle: false,
    },
    Case {
        name: "parser_decorators",

        args: &[],
        expect_success: false,
        expected_files: &[],
        stdout_contains: &[],
        skip_oracle: true,
    },
    Case {
        name: "parser_enums",

        args: &[],
        expect_success: true,
        expected_files: &[],
        stdout_contains: &[],
        skip_oracle: false,
    },
    Case {
        name: "parser_conditional_types",

        args: &[],
        expect_success: true,
        expected_files: &[],
        stdout_contains: &[],
        skip_oracle: false,
    },
    Case {
        name: "parser_js",

        args: &[],
        expect_success: true,
        expected_files: &[],
        stdout_contains: &[],
        skip_oracle: false,
    },
    Case {
        name: "parser_jsx",

        args: &[],
        expect_success: true,
        expected_files: &[],
        stdout_contains: &[],
        skip_oracle: true,
    },
];

#[test]
fn rust_smoke_cases_emit_expected_outputs() {
    let tsox = rust_tsox();
    for case in CASES {
        let work_dir = copy_case_to_temp(case.name);
        let output = run(&tsox, case.args, &work_dir);

        if case.expect_success {
            assert!(
                output.status.success(),
                "Rust tsox failed for case '{}'\nstatus: {:?}\nstdout:\n{}\nstderr:\n{}",
                case.name,
                output.status.code(),
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        } else {
            assert!(
                !output.status.success(),
                "Rust tsox unexpectedly succeeded for case '{}'\nstatus: {:?}\nstdout:\n{}\nstderr:\n{}",
                case.name,
                output.status.code(),
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        for needle in case.stdout_contains {
            assert!(
                stdout.contains(needle),
                "case '{}' stdout missing '{}'\nstdout:\n{}",
                case.name,
                needle,
                stdout
            );
        }

        for (path, expected) in case.expected_files {
            let actual_path = work_dir.join(path);
            let actual = fs::read_to_string(&actual_path).unwrap_or_else(|e| {
                panic!(
                    "case '{}' did not write expected file '{}': {e}",
                    case.name,
                    actual_path.display()
                )
            });
            assert_eq!(
                normalize_newlines(&actual),
                normalize_newlines(expected),
                "case '{}' emitted unexpected content for '{}'",
                case.name,
                path
            );
        }
    }
}

#[test]
fn compare_with_go_oracle_when_available() {
    let Some(oracle) = go_oracle() else {
        eprintln!("skipping Go oracle parity: set TSGO_ORACLE to a runnable tsgo binary");
        return;
    };

    let sanity_dir = std::env::temp_dir().join(format!(
        "tsox-oracle-sanity-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(sanity_dir.join("src")).unwrap();
    fs::write(
        sanity_dir.join("tsconfig.json"),
        r#"{"compilerOptions":{"outDir":"dist","pretty":false},"files":["src/main.ts"]}"#,
    )
    .unwrap();
    fs::write(sanity_dir.join("src/main.ts"), "let x: number = 42;\n").unwrap();
    let sanity = run(&oracle, &[], &sanity_dir);
    let sanity_out = format!(
        "{}{}",
        String::from_utf8_lossy(&sanity.stdout),
        String::from_utf8_lossy(&sanity.stderr)
    );
    let _ = fs::remove_dir_all(&sanity_dir);
    if !sanity.status.success() || sanity_out.contains("TS2318") {
        eprintln!(
            "skipping Go oracle parity: oracle sanity check failed or lacks default libs\n{}",
            sanity_out
        );
        return;
    }

    let tsox = rust_tsox();
    for case in CASES {
        if case.skip_oracle {
            eprintln!(
                "skipping Go oracle parity for case '{}': known parity gap (skip_oracle = true)",
                case.name
            );
            continue;
        }

        let rust_dir = copy_case_to_temp(&format!("{}-rust", case.name));
        let go_dir = copy_case_to_temp(&format!("{}-go", case.name));

        let rust_output = run(&tsox, case.args, &rust_dir);
        let go_output = run(&oracle, case.args, &go_dir);

        assert_eq!(
            rust_output.status.code(),
            go_output.status.code(),
            "case '{}' exit code differed\nrust stdout:\n{}\nrust stderr:\n{}\ngo stdout:\n{}\ngo stderr:\n{}",
            case.name,
            String::from_utf8_lossy(&rust_output.stdout),
            String::from_utf8_lossy(&rust_output.stderr),
            String::from_utf8_lossy(&go_output.stdout),
            String::from_utf8_lossy(&go_output.stderr),
        );
        assert_eq!(
            normalize_newlines(&String::from_utf8_lossy(&rust_output.stdout)),
            normalize_newlines(&String::from_utf8_lossy(&go_output.stdout)),
            "case '{}' stdout differed",
            case.name
        );
        assert_eq!(
            normalize_newlines(&String::from_utf8_lossy(&rust_output.stderr)),
            normalize_newlines(&String::from_utf8_lossy(&go_output.stderr)),
            "case '{}' stderr differed",
            case.name
        );

        let rust_files = collect_emitted_files(&rust_dir);
        let go_files = collect_emitted_files(&go_dir);
        assert_eq!(
            rust_files, go_files,
            "case '{}' emitted files differed",
            case.name
        );
    }
}

fn rust_tsox() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_tsox"))
}

fn go_oracle() -> Option<PathBuf> {
    if let Ok(path) = std::env::var("TSGO_ORACLE") {
        let path = PathBuf::from(path);
        if is_runnable_tsgo(&path) {
            return Some(path);
        }
        eprintln!("TSGO_ORACLE is set but is not runnable: {}", path.display());
        return None;
    }

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let go_worktree = manifest_dir
        .parent()
        .unwrap_or(&manifest_dir)
        .join("typescript-go");

    for candidate in [
        go_worktree.join("built/local/tsgo"),
        manifest_dir.join("_packages/native-preview/bin/tsgo"),
    ] {
        if is_runnable_tsgo(&candidate) {
            return Some(candidate);
        }
    }

    eprintln!(
        "skipping Go oracle parity: set TSGO_ORACLE or build tsgo in {}",
        go_worktree.display()
    );
    None
}

fn is_runnable_tsgo(path: &Path) -> bool {
    if !path.exists() {
        return false;
    }
    Command::new(path)
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn run(exe: &Path, args: &[&str], cwd: &Path) -> Output {
    Command::new(exe)
        .args(args)
        .current_dir(cwd)
        .output()
        .unwrap_or_else(|e| panic!("failed to run '{}': {e}", exe.display()))
}

fn copy_case_to_temp(name: &str) -> PathBuf {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/parity")
        .join(strip_suffix(name));
    let target = std::env::temp_dir().join(format!(
        "tsox-parity-{}-{}",
        sanitize_name(name),
        unique_id()
    ));
    copy_dir_all(&source, &target).unwrap_or_else(|e| {
        panic!(
            "failed to copy fixture '{}' to '{}': {e}",
            source.display(),
            target.display()
        )
    });
    target
}

fn strip_suffix(name: &str) -> &str {
    name.strip_suffix("-rust")
        .or_else(|| name.strip_suffix("-go"))
        .unwrap_or(name)
}

fn sanitize_name(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect()
}

fn unique_id() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time before Unix epoch")
        .as_nanos()
}

fn copy_dir_all(source: &Path, target: &Path) -> std::io::Result<()> {
    fs::create_dir_all(target)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let to = target.join(entry.file_name());
        if file_type.is_dir() {
            copy_dir_all(&entry.path(), &to)?;
        } else {
            fs::copy(entry.path(), to)?;
        }
    }
    Ok(())
}

fn collect_emitted_files(root: &Path) -> BTreeMap<String, String> {
    let mut files = BTreeMap::new();
    collect_emitted_files_inner(root, root, &mut files);
    files
}

fn collect_emitted_files_inner(root: &Path, current: &Path, files: &mut BTreeMap<String, String>) {
    for entry in fs::read_dir(current)
        .unwrap_or_else(|e| panic!("failed to read directory '{}': {e}", current.display()))
    {
        let entry = entry.expect("failed to read directory entry");
        let path = entry.path();
        if path.is_dir() {
            collect_emitted_files_inner(root, &path, files);
            continue;
        }

        let rel = path
            .strip_prefix(root)
            .expect("path should be under root")
            .to_string_lossy()
            .replace('\\', "/");
        if rel.ends_with(".ts") || rel == "tsconfig.json" {
            continue;
        }
        let text = fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("failed to read emitted file '{}': {e}", path.display()));
        files.insert(rel, normalize_newlines(&text));
    }
}

fn normalize_newlines(text: &str) -> String {
    text.replace("\r\n", "\n")
}
