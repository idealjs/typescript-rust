use super::*;
use crate::bundled::lib_path;
use crate::compiler::{CompilerHost, CompilerHostImpl, Program, ProgramOptions};
use crate::tsoptions::ParsedCommandLine;
use crate::vfs::InMemoryFS;

fn build_checker_with_lib(source: &str) -> Checker {
    use crate::bundled::BundledFS;
    let inner = Arc::new(InMemoryFS::new());
    inner.insert_file("/proj/entry.ts", source);
    inner.insert_file(
        "/proj/tsconfig.json",
        "{ \"compilerOptions\": {}, \"files\": [\"entry.ts\"] }",
    );
    let fs = Arc::new(BundledFS::new(inner));
    let parsed = ParsedCommandLine {
        file_names: vec!["/proj/entry.ts".to_string()],
        ..Default::default()
    };
    let host: Arc<dyn CompilerHost> =
        Arc::new(CompilerHostImpl::new(fs, "/proj".to_string(), lib_path()));
    let program = Arc::new(Program::new(ProgramOptions {
        config: parsed,
        host,
    }));
    program.build_checker()
}

fn error_codes(checker: &Checker) -> Vec<i32> {
    let codes: Vec<i32> = checker
        .diagnostics
        .get_all()
        .iter()
        .filter(|d| {
            !d.file
                .as_ref()
                .is_some_and(|f| f.file_name.starts_with("bundled://"))
        })
        .map(|d| d.code)
        .collect();
    codes
}

#[test]
fn array_every_callback_param_typed_by_element() {
    let ok = build_checker_with_lib("declare const ss: string[]; ss.every((x: string) => true);");
    assert_eq!(
        error_codes(&ok),
        Vec::<i32>::new(),
        "matching callback must pass"
    );

    let bad = build_checker_with_lib("declare const ss: string[]; ss.every((x: number) => true);");

    assert_eq!(
        error_codes(&bad),
        vec![2769],
        "mismatched callback param must fail"
    );
}

#[test]
fn array_flat_own_type_params_stay_free() {
    let ok = build_checker_with_lib(
        "function foo<T>(arr: T[], depth: number) { return arr.flat(depth); }",
    );
    assert_eq!(error_codes(&ok), Vec::<i32>::new());
}

#[test]
fn array_method_signature_display_substituted() {
    let checker = build_checker_with_lib("declare const ss: string[]; ss.every(42);");
    let codes = super::convergence_tests::error_codes(&checker);
    assert_eq!(codes, vec![2769]);
    let diag = checker
        .diagnostics
        .get_all()
        .iter()
        .find(|d| d.code == 2769)
        .cloned()
        .unwrap();
    let template = diag.message.as_ref().map(|m| m.text).unwrap_or("");
    let mut msg = template.to_string();
    for (i, a) in diag.message_args.iter().enumerate() {
        msg = msg.replace(&format!("{{{i}}}"), a);
    }

    fn collect_chain_text(d: &crate::ast::Diagnostic, out: &mut String) {
        out.push_str(&d.message.as_ref().map(|m| m.text).unwrap_or(""));
        for (i, a) in d.message_args.iter().enumerate() {
            out.push(' ');
            out.push_str(a);
            let _ = i;
        }
        for c in &d.message_chain {
            collect_chain_text(c, out);
        }
    }
    let mut full = msg;
    collect_chain_text(&diag, &mut full);
    assert!(
        full.contains("(value: string, index: number, array: string[])"),
        "message should show the element-substituted signature: {full}"
    );
}

#[test]
fn explicit_type_arguments_select_generic_overload() {
    let ok = build_checker_with_lib(
        "declare const a: string[]; const r = a.reduce<number>((c, d) => c + d, \" \");",
    );
    assert_eq!(error_codes(&ok), Vec::<i32>::new());
}

#[test]
fn bare_array_assignable_to_concat_array() {
    let ok = build_checker_with_lib(
        "declare const a: string[]; const c: ConcatArray<string> = a; const r = a.concat(\"x\");",
    );
    assert_eq!(error_codes(&ok), Vec::<i32>::new());
}

#[test]
fn concat_on_number_array_with_array_arg() {
    let ok = build_checker_with_lib("declare const fa: number[]; var x = fa.concat(fa);");
    assert_eq!(error_codes(&ok), Vec::<i32>::new());
}
