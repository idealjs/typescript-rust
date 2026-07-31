use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug)]
struct Case {
    name: &'static str,
    args: &'static [&'static str],
    expected_files: &'static [(&'static str, &'static str)],
}

const CASES: &[Case] = &[
    Case {
        name: "simple_emit",
        args: &[],
        expected_files: &[("dist/main.js", "let answer = 42;\n")],
    },
    Case {
        name: "type_only_declarations",
        args: &[],
        expected_files: &[("dist/main.js", "const value = { name: \"Ada\" };\n")],
    },
    Case {
        name: "nested_out_dir",
        args: &[],
        // This documents the current Rust behavior. The Go oracle comparison
        // will expose that outputpaths still need proper rootDir/outDir parity.
        expected_files: &[(
            "dist/main.js",
            "export function square(x) { return x * x; }\n",
        )],
    },
];

#[test]
fn rust_smoke_cases_emit_expected_outputs() {
    let tsox = rust_tsox();
    for case in CASES {
        let work_dir = copy_case_to_temp(case.name);
        let output = run(&tsox, case.args, &work_dir);
        assert!(
            output.status.success(),
            "Rust tsox failed for case '{}'\nstatus: {:?}\nstdout:\n{}\nstderr:\n{}",
            case.name,
            output.status.code(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );

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

    let tsox = rust_tsox();
    for case in CASES {
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
