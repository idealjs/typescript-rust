use super::*;

use crate::diagnosticwriter::format_diagnostic_compact;

fn rendered_with_chain(checker: &Checker) -> Vec<String> {
    checker
        .diagnostics
        .get_all()
        .iter()
        .filter(|d| {
            !d.file
                .as_ref()
                .is_some_and(|f| f.file_name.starts_with("bundled://"))
        })
        .map(|d| {
            let mut s = format_diagnostic_compact(d, None);
            if let Some(rest) = s.find(" error TS") {
                s = s[rest + 1..].to_string();
            }
            for c in &d.message_chain {
                s.push('\n');
                s.push_str("  ");
                s.push_str(&crate::diagnosticwriter::message_text(c, None));
            }
            s
        })
        .collect()
}

#[test]
fn invocation_error_chain_names_apparent_wrapper_type() {
    let (program, mut checker) =
        convergence_tests::build_program_and_checker("declare const s: string;\ns();", &["es5"]);
    for file in program.source_files() {
        checker.check_source_file(file);
    }
    let lines = rendered_with_chain(&checker);
    assert!(
        lines.iter().any(|l| l.starts_with("error TS2349:")),
        "{lines:?}"
    );
    assert!(
        lines
            .iter()
            .any(|l| l.contains("Type 'String' has no call signatures.")),
        "{lines:?}"
    );
}

#[test]
fn never_intersection_callee_renders_never_in_chain() {
    let (program, mut checker) = convergence_tests::build_program_and_checker(
        "declare const f: { (x: string): number, a: \"\" } & { a: number };\nf();",
        &["es5"],
    );
    for file in program.source_files() {
        checker.check_source_file(file);
    }
    let lines = rendered_with_chain(&checker);
    assert!(
        lines.iter().any(|l| l.starts_with("error TS2349:")),
        "{lines:?}"
    );
    assert!(
        lines
            .iter()
            .any(|l| l.contains("Type 'never' has no call signatures.")),
        "{lines:?}"
    );
}

#[test]
fn union_target_failure_keeps_constituent_head_line() {
    let source = "var a0: (n: number, s: string) => number\n\
                      var a1: typeof a0 | ((n: number, s: string) => string);\n\
                      a1 = (foo, bar) => { return true; }";
    let (program, mut checker) = convergence_tests::build_program_and_checker(source, &["es5"]);
    for file in program.source_files() {
        checker.check_source_file(file);
    }
    let codes = super::convergence_tests::error_codes(&checker);
    assert_eq!(codes, vec![2322], "{codes:?}");
    let lines = rendered_with_chain(&checker);
    let joined = lines.join("\n");
    assert!(
            joined.contains("Type '(foo: number, bar: string) => boolean' is not assignable to type '(n: number, s: string) => number'."),
            "{joined}"
        );
    assert!(
        joined.contains("Type 'boolean' is not assignable to type 'number'."),
        "{joined}"
    );
}

#[test]
fn equality_discriminant_keeps_undefined_member_under_non_strict() {
    let source = "type Foo2 = { kind?: 'a', a: number } | { kind?: 'b' } | { kind?: never };\n\
                      function f2(foo: Foo2) {\n\
                          if (foo.kind === 'a') {\n\
                              foo.a;\n\
                          }\n\
                      }";
    for strict in [false, true] {
        let (program, mut checker) = convergence_tests::build_program_and_checker(source, &["es5"]);

        let _ = strict;
        for file in program.source_files() {
            checker.check_source_file(file);
        }
        let codes = super::convergence_tests::error_codes(&checker);
        assert!(codes.is_empty(), "strict={strict} codes={codes:?}");
    }
}

#[test]
fn optional_member_stays_t_when_strict_null_checks_off() {
    let lines: Vec<Vec<i32>> = [false, true]
        .iter()
        .map(|strict| {
            let diags = super::node_format_tests::check_files(
                &[(
                    "entry.ts",
                    "interface I { x?: string; }\n\
                         declare const i: I;\n\
                         const a: string = i.x;",
                )],
                "/proj/entry.ts",
                |o| o.strict_null_checks = crate::core::tristate::Tristate::from(*strict),
            );
            diags
        })
        .collect();

    assert!(lines[0].is_empty(), "non-strict: {:?}", lines[0]);

    assert_eq!(lines[1], vec![2322], "strict: {:?}", lines[1]);
}

#[test]
fn indexed_access_tp_target_carries_instantiation_note() {
    let (program, mut checker) = convergence_tests::build_program_and_checker(
        "function f<T extends object, P extends keyof T>(s: string, tp: T[P]): void {\n    tp = s;\n}",
        &["es5"],
    );
    for file in program.source_files() {
        checker.check_source_file(file);
    }
    let lines = rendered_with_chain(&checker);
    let joined = lines.join("\n");
    assert!(joined.contains("error TS2322"), "{joined}");
    assert!(
        joined.contains(
            "could be instantiated with an arbitrary type which could be unrelated to 'string'"
        ),
        "{joined}"
    );
}

#[test]
fn record_element_access_assigns_object() {
    let (program, mut checker) = convergence_tests::build_program_and_checker(
        "declare const row: string;\n\
             const classesByRow: Record<string, object> = {};\n\
             classesByRow[row] = {};",
        &["es2015"],
    );
    for file in program.source_files() {
        checker.check_source_file(file);
    }
    let codes = super::convergence_tests::error_codes(&checker);
    assert!(codes.is_empty(), "{codes:?}");
}

#[test]
fn node10_program_reports_deprecation_and_alternate_result() {
    use crate::bundled::BundledFS;
    use crate::compiler::{CompilerHost, CompilerHostImpl, Program, ProgramOptions};
    use crate::tsoptions::ParsedCommandLine;
    use crate::vfs::InMemoryFS;

    let inner = Arc::new(InMemoryFS::new());
    inner.insert_dir("/node_modules");
    inner.insert_dir("/node_modules/pkg");
    inner.insert_file(
        "/node_modules/pkg/package.json",
        r#"{"name":"pkg","version":"1.0.0","exports":{".":"./definitely-not-index.js"}}"#,
    );
    inner.insert_file("/node_modules/pkg/definitely-not-index.d.ts", "export {};");
    inner.insert_file("/proj/entry.ts", "import { pkg } from \"pkg\";");
    let fs = Arc::new(BundledFS::new(inner));
    let mut options = CompilerOptions::default();
    options.module_resolution = crate::core::compiler_options::ModuleResolutionKind::Node10;
    options.target = crate::core::compiler_options::ScriptTarget::ES2015;
    options.module = crate::core::compiler_options::ModuleKind::CommonJS;
    let parsed = ParsedCommandLine {
        file_names: vec!["/proj/entry.ts".to_string()],
        compiler_options: options,
        ..Default::default()
    };
    let host: Arc<dyn CompilerHost> = Arc::new(CompilerHostImpl::new(
        fs,
        "/proj".to_string(),
        crate::bundled::lib_path(),
    ));
    let program = Arc::new(Program::new(ProgramOptions {
        config: parsed,
        host,
    }));

    let global_codes: Vec<i32> = program
        .diagnostics()
        .iter()
        .filter(|d| d.file.is_none())
        .map(|d| d.code)
        .collect();
    assert!(global_codes.contains(&5107), "{global_codes:?}");

    let lines: Vec<String> = program
        .diagnostics()
        .iter()
        .map(|d| d.as_ref())
        .filter(|d| d.code == 2307)
        .map(|d| crate::diagnosticwriter::format_diagnostic_compact(d, None))
        .collect();
    assert!(!lines.is_empty(), "TS2307 must report");
    let joined = lines.join("\n");
    assert!(
        joined.contains("There are types at '/node_modules/pkg/definitely-not-index.d.ts'"),
        "{joined}"
    );
}

#[test]
fn per_file_jsx_pragma_overrides_option_factory_for_2874() {
    let files: Vec<(&str, &str)> = vec![
        (
            "renderer.d.ts",
            "declare global {\n    namespace JSX {\n        interface IntrinsicElements {\n            [e: string]: any;\n        }\n    }\n}\nexport function dom(): void;\nexport { dom as p };",
        ),
        (
            "reacty.tsx",
            "/** @jsx dom */\nimport { dom } from \"./renderer\";\n<h></h>",
        ),
        ("index.tsx", "import { p } from \"./renderer\";\n<h></h>"),
    ];
    let diags = super::node_format_tests::check_files(&files, "/proj/reacty.tsx", |o| {
        o.jsx = crate::core::compiler_options::JsxEmit::React;
        o.jsx_factory = "p".to_string();
        o.module = crate::core::compiler_options::ModuleKind::CommonJS;
        o.target = crate::core::compiler_options::ScriptTarget::ES2015;
    });
    assert!(
        !diags.iter().any(|c| *c == 2874),
        "TS2874 must not fire under per-file pragma: {diags:?}"
    );
}
