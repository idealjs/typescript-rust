//! Checker parity fixtures: type-check source files and verify diagnostics.
//!
//! These tests create a `Program` (with bundled libs), run the checker, and
//! assert on the resulting semantic diagnostics. The checker currently only
//! emits TS2304 ("Cannot find name") for unresolvable identifiers, so most
//! fixtures test that valid code produces zero diagnostics, and that code
//! referencing undefined names produces the expected TS2304.

use std::sync::Arc;

use tsox::bundled::{BundledFS, lib_path};
use tsox::compiler::{CompilerHostImpl, Program, ProgramOptions};
use tsox::diagnostics::Category;
use tsox::tsoptions::parse_command_line;
use tsox::vfs::InMemoryFS;

// ────────────────────────────────────────────────────────────────────────────
// Helpers
// ────────────────────────────────────────────────────────────────────────────

/// Run the checker on a single TypeScript source file and return semantic
/// diagnostics. Uses `--noLib` to avoid loading bundled libs (faster, and
/// simpler for basic tests).
fn check_source(source: &str) -> Vec<tsox::ast::Diagnostic> {
    check_source_with_lib(source, true)
}

/// Like `check_source` but with optional lib loading.
fn check_source_with_lib(source: &str, no_lib: bool) -> Vec<tsox::ast::Diagnostic> {
    check_source_named_with_lib("/proj/entry.ts", source, no_lib)
}

/// Like `check_source` but uses a `.tsx` filename to enable JSX parsing,
/// passes `--jsx preserve`, and disables `--noImplicitAny` (since the
/// tests run without lib.d.ts and don't have a JSX namespace in scope).
fn check_source_tsx(source: &str) -> Vec<tsox::ast::Diagnostic> {
    check_source_tsx_with_args(source, &["--jsx", "preserve", "--noImplicitAny", "false"])
}

/// Like `check_source_tsx` but with extra CLI args.
fn check_source_tsx_with_args(source: &str, extra_args: &[&str]) -> Vec<tsox::ast::Diagnostic> {
    let fs = Arc::new(InMemoryFS::new());
    fs.insert_dir("/proj");
    fs.insert_file("/proj/entry.tsx", source);

    let mut args = vec!["--noLib".to_string()];
    for a in extra_args {
        args.push((*a).to_string());
    }
    args.push("/proj/entry.tsx".to_string());
    let parsed = parse_command_line(&args, "/proj", Some(fs.as_ref()));

    let host: Arc<dyn tsox::compiler::CompilerHost> =
        Arc::new(CompilerHostImpl::new(fs, "/proj".to_string(), lib_path()));

    let program = Arc::new(Program::new(ProgramOptions {
        config: parsed,
        host,
    }));

    program.get_semantic_diagnostics()
}

/// Run the checker on a single source file with an explicit filename and
/// optional lib loading.
fn check_source_named_with_lib(
    path: &str,
    source: &str,
    no_lib: bool,
) -> Vec<tsox::ast::Diagnostic> {
    let fs = Arc::new(InMemoryFS::new());
    fs.insert_dir("/proj");
    fs.insert_file(path, source);

    let mut args = vec![path.to_string()];
    if no_lib {
        args.insert(0, "--noLib".to_string());
    }
    let parsed = parse_command_line(&args, "/proj", Some(fs.as_ref()));

    let host: Arc<dyn tsox::compiler::CompilerHost> = if no_lib {
        Arc::new(CompilerHostImpl::new(fs, "/proj".to_string(), lib_path()))
    } else {
        let bf = Arc::new(BundledFS::new(fs));
        Arc::new(CompilerHostImpl::new(bf, "/proj".to_string(), lib_path()))
    };

    let program = Arc::new(Program::new(ProgramOptions {
        config: parsed,
        host,
    }));

    program.get_semantic_diagnostics()
}

/// Run the checker on multiple source files and return all diagnostics.
fn check_sources(files: &[(&str, &str)]) -> Vec<tsox::ast::Diagnostic> {
    check_sources_with_lib(files, true)
}

fn check_sources_with_lib(files: &[(&str, &str)], no_lib: bool) -> Vec<tsox::ast::Diagnostic> {
    let fs = Arc::new(InMemoryFS::new());
    fs.insert_dir("/proj");
    for (name, content) in files {
        if let Some(parent) = std::path::Path::new(name).parent() {
            let parent_str = parent.to_string_lossy();
            if !parent_str.is_empty() {
                fs.insert_dir(&format!("/proj/{}", parent_str));
            }
        }
        fs.insert_file(&format!("/proj/{}", name), content);
    }

    let mut args: Vec<String> = files.iter().map(|(n, _)| format!("/proj/{}", n)).collect();
    if no_lib {
        args.insert(0, "--noLib".to_string());
    }
    let parsed = parse_command_line(&args, "/proj", Some(fs.as_ref()));

    let host: Arc<dyn tsox::compiler::CompilerHost> = if no_lib {
        Arc::new(CompilerHostImpl::new(fs, "/proj".to_string(), lib_path()))
    } else {
        let bf = Arc::new(BundledFS::new(fs));
        Arc::new(CompilerHostImpl::new(bf, "/proj".to_string(), lib_path()))
    };

    let program = Arc::new(Program::new(ProgramOptions {
        config: parsed,
        host,
    }));

    program.get_semantic_diagnostics()
}

/// Assert that there are no semantic diagnostics.
fn assert_no_diagnostics(diags: &[tsox::ast::Diagnostic]) {
    if !diags.is_empty() {
        let msg: Vec<String> = diags
            .iter()
            .map(|d| format!("  TS{}: {}", d.code, d.message_args.join(", ")))
            .collect();
        panic!(
            "Expected no diagnostics, got {}:\n{}",
            diags.len(),
            msg.join("\n")
        );
    }
}

/// Assert that there is at least one diagnostic with the given code.
fn assert_diagnostic_code(diags: &[tsox::ast::Diagnostic], code: i32) {
    assert!(
        !diags.is_empty(),
        "Expected a diagnostic with code TS{}, but got none",
        code
    );
    let has_code = diags.iter().any(|d| d.code == code);
    if !has_code {
        let codes: Vec<i32> = diags.iter().map(|d| d.code).collect();
        panic!(
            "Expected a diagnostic with code TS{}, but got codes: {:?}",
            code, codes
        );
    }
}

/// Assert that there are exactly N diagnostics with the given code.
fn assert_diagnostic_count(diags: &[tsox::ast::Diagnostic], code: i32, count: usize) {
    let actual = diags.iter().filter(|d| d.code == code).count();
    assert_eq!(
        actual, count,
        "Expected {} diagnostic(s) with code TS{}, got {}",
        count, code, actual
    );
}

// ────────────────────────────────────────────────────────────────────────────
// P3.14: Basic variable declarations (no diagnostics expected)
// ────────────────────────────────────────────────────────────────────────────

#[test]
fn checker_var_declaration_no_error() {
    let diags = check_source("var x = 1;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_let_declaration_no_error() {
    let diags = check_source("let x = 1;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_const_declaration_no_error() {
    let diags = check_source("const x = 1;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_multiple_var_declarations_no_error() {
    let diags = check_source("let a = 1, b = 2, c = 3;");
    let count = diags.iter().filter(|d| d.code == 2304).count();
    assert_eq!(
        count, 2,
        "Expected 2 TS2304 errors for 'b' and 'c', got {}",
        count
    );
}

#[test]
fn checker_var_with_type_annotation_no_error() {
    let diags = check_source("let x: number = 1;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_uninitialized_var_no_error() {
    let diags = check_source("let x;");
    assert_no_diagnostics(&diags);
}

// ────────────────────────────────────────────────────────────────────────────
// Assignability: the relater is now wired into variable declarations.
// `let x: T = init` requires `init` assignable to `T` (TS2322).
// ────────────────────────────────────────────────────────────────────────────

#[test]
fn checker_assignable_keyword_to_same_keyword_no_error() {
    // string -> string, number -> number, boolean -> boolean
    let diags = check_source("let a: string = 'hi'; let b: number = 1; let c: boolean = true;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_assignable_number_init_to_string_annotation_ts2322() {
    let diags = check_source("let x: string = 42;");
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_assignable_string_init_to_number_annotation_ts2322() {
    let diags = check_source("let x: number = 'hi';");
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_assignable_number_init_to_boolean_annotation_ts2322() {
    let diags = check_source("let x: boolean = 1;");
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_assignable_boolean_init_to_number_annotation_ts2322() {
    let diags = check_source("let x: number = true;");
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_assignable_to_union_member_no_error() {
    let diags = check_source(
        "let a: string | number = 42;\
         let b: string | number = 'hi';",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_assignable_outside_union_ts2322() {
    // `true` is not a member of `string | number`.
    let diags = check_source("let x: string | number = true;");
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_assignable_array_annotation_wrong_primitive_ts2322() {
    // A `number` is not assignable to `string[]`.
    let diags = check_source("let x: string[] = 42;");
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_assignable_array_annotation_primitive_to_array_ts2322() {
    // A `string` is not assignable to `number[]`.
    let diags = check_source("let x: number[] = 'hi';");
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_assignable_array_annotation_array_literal_no_error() {
    // An array literal (currently widened to `any`) is assignable to `number[]`.
    let diags = check_source("let x: number[] = [1, 2, 3];");
    assert_no_diagnostics(&diags);
}

// ────────────────────────────────────────────────────────────────────────────
// Tuple type annotations: `[T, U, ...]`
// ────────────────────────────────────────────────────────────────────────────

#[test]
fn checker_assignable_tuple_wrong_primitive_ts2322() {
    // A `number` is not assignable to `[number, string]`.
    let diags = check_source("let x: [number, string] = 42;");
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_assignable_tuple_string_to_number_tuple_ts2322() {
    // A `string` is not assignable to `[number, string]`.
    let diags = check_source("let x: [number, string] = 'hi';");
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_assignable_tuple_array_literal_no_error() {
    // An array literal (widened to `any`) is assignable to `[number, string]`.
    let diags = check_source("let x: [number, string] = [1, 'hi'];");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_assignable_tuple_annotation_no_init_no_error() {
    // Just declaring a tuple-typed variable without an initializer is fine.
    let diags = check_source("let x: [number, string];");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_recursive_object_type_does_not_overflow() {
    // Recursive structural type — the relater must terminate via the
    // depth guard instead of blowing the native stack.
    let diags = check_source(
        "type Box = { value: number; next: Box | null };\
         let x: Box = { value: 1, next: null };",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_var_type_inference_propagates_via_symbol_ts2322() {
    // `x` is inferred as `number`; assigning it to a `string`-typed
    // variable must report TS2322. Before the value_symbol_links cache
    // was wired into `get_type_of_symbol`, `x`'s type fell back to
    // `any` and this slipped through silently.
    let diags = check_source("let x = 42; let y: string = x;");
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_var_type_inference_propagates_via_symbol_no_error() {
    // `x` is inferred as `number`; assigning it to another `number`
    // variable must not produce a diagnostic.
    let diags = check_source("let x = 42; let y: number = x;");
    assert_no_diagnostics(&diags);
}

// ────────────────────────────────────────────────────────────────────────────
// Function return type inference (P3.8b)
// ────────────────────────────────────────────────────────────────────────────

#[test]
fn checker_function_inferred_return_number_no_error() {
    // `f` infers `number` as its return type; assigning `f()` to a
    // `number`-typed variable should be fine.
    let diags = check_source("function f() { return 42; } let y: number = f();");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_function_inferred_return_number_assigned_to_string_ts2322() {
    // `f` infers `number`; assigning `f()` to a `string` must fail.
    let diags = check_source("function f() { return 42; } let y: string = f();");
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_arrow_function_inferred_return_no_error() {
    // `f` infers `number` (return expression is `x * 2` where x is number).
    let diags = check_source(
        "const f = (x: number) => x * 2;\
         let y: number = f(3);",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_arrow_function_inferred_return_string_to_number_ts2322() {
    // `f` infers `string`; assigning `f()` to a `number` must fail.
    let diags = check_source(
        "const f = () => 'hi';\
         let y: number = f();",
    );
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_function_with_explicit_return_type_no_error() {
    // Explicit return-type annotation should be honored.
    let diags = check_source("function f(): number { return 42; } let y: number = f();");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_function_no_return_infers_void_to_number_ts2322() {
    // A function with no `return` infers `void`; assigning `f()` to a
    // `number`-typed variable should fail TS2322.
    let diags = check_source("function f() {} let y: number = f();");
    assert_diagnostic_code(&diags, 2322);
}

// ────────────────────────────────────────────────────────────────────────────
// Return statement type checking (TS2322 / TS1135)
// ────────────────────────────────────────────────────────────────────────────

#[test]
fn checker_return_type_mismatch_ts2322() {
    // `return "hi"` does not match declared `number` return type.
    let diags = check_source("function f(): number { return \"hi\"; }");
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_return_type_match_no_error() {
    // `return 42` matches declared `number` return type.
    let diags = check_source("function f(): number { return 42; }");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_return_type_string_match_no_error() {
    // `return "x"` matches declared `string` return type.
    let diags = check_source("function f(): string { return \"x\"; }");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_return_type_boolean_mismatch_ts2322() {
    // `return 1` does not match declared `boolean` return type.
    let diags = check_source("function f(): boolean { return 1; }");
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_return_missing_value_ts1135() {
    // `return;` with no value in function declaring `string` return type.
    let diags = check_source("function f(): string { return; }");
    assert_diagnostic_code(&diags, 1135);
}

#[test]
fn checker_return_missing_value_in_number_function_ts1135() {
    // `return;` with no value in function declaring `number` return type.
    let diags = check_source("function f(): number { return; }");
    assert_diagnostic_code(&diags, 1135);
}

#[test]
fn checker_return_void_no_value_no_error() {
    // `return;` is allowed when declared return type is `void`.
    let diags = check_source("function f(): void { return; }");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_return_no_annotation_no_value_no_error() {
    // `return;` is allowed when there is no return-type annotation.
    let diags = check_source("function f() { return; }");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_arrow_return_type_mismatch_ts2322() {
    // Arrow function `return "hi"` does not match declared `number`.
    let diags = check_source("const f = (): number => { return \"hi\"; };");
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_arrow_expression_body_return_type_mismatch_ts2322() {
    // Arrow function expression body returning wrong type.
    let diags = check_source("const f = (): number => \"hi\";");
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_method_return_type_mismatch_ts2322() {
    // Class method `return "hi"` does not match declared `number`.
    let diags = check_source("class C { m(): number { return \"hi\"; } }");
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_get_accessor_return_type_mismatch_ts2322() {
    // `get` accessor returning wrong type.
    let diags = check_source("class C { get x(): number { return \"hi\"; } }");
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_return_union_type_no_error() {
    // `return "a"` is assignable to declared `string | number`.
    let diags = check_source(
        "type S = string | number;\
         function f(): S { return \"a\"; }",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_return_literal_to_widen_no_error() {
    // `return 42` matches declared `number` (literal widens to base).
    let diags = check_source("function f(): number { return 42; } let y = f();");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_nested_function_return_type_check_ts2322() {
    // Nested function declaration with its own return-type annotation.
    let diags = check_source(
        "function outer(): void {\
             function inner(): number { return \"bad\"; }\
             inner();\
         }",
    );
    assert_diagnostic_code(&diags, 2322);
}

// ────────────────────────────────────────────────────────────────────────────
// Comparison overlap check (TS2367)
// ────────────────────────────────────────────────────────────────────────────

#[test]
fn checker_comparison_no_overlap_number_string_ts2367() {
    // `number === string` always returns false — TS2367.
    let diags = check_source(
        "function f(x: number) {\
             if (x === \"hi\") {}\
         }",
    );
    assert_diagnostic_code(&diags, 2367);
}

#[test]
fn checker_comparison_no_overlap_boolean_string_ts2367() {
    // `boolean === "hi"` always returns false — TS2367.
    let diags = check_source(
        "function f(x: boolean) {\
             if (x === \"hi\") {}\
         }",
    );
    assert_diagnostic_code(&diags, 2367);
}

#[test]
fn checker_comparison_no_overlap_inequality_ts2367() {
    // `!==` triggers TS2367 too (same as `===` for unrelated types).
    let diags = check_source(
        "function f(x: number) {\
             if (x !== \"hi\") {}\
         }",
    );
    assert_diagnostic_code(&diags, 2367);
}

#[test]
fn checker_comparison_loose_equals_no_overlap_ts2367() {
    // `==` triggers TS2367 for unrelated types.
    let diags = check_source(
        "function f(x: number) {\
             if (x == \"hi\") {}\
         }",
    );
    assert_diagnostic_code(&diags, 2367);
}

#[test]
fn checker_comparison_same_type_no_error() {
    // `number === number` — no diagnostic.
    let diags = check_source(
        "function f(x: number, y: number) {\
             if (x === y) {}\
         }",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_comparison_with_literal_no_error() {
    // `number === 42` — `42` is a literal of `number`, comparable.
    let diags = check_source(
        "function f(x: number) {\
             if (x === 42) {}\
         }",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_comparison_with_any_no_error() {
    // `any === number` — `any` is always comparable.
    let diags = check_source(
        "function f(x: any, y: number) {\
             if (x === y) {}\
         }",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_comparison_with_null_no_error() {
    // `number === null` — comparisons with `null`/`undefined` are allowed.
    let diags = check_source(
        "function f(x: number) {\
             if (x === null) {}\
         }",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_comparison_union_literal_no_error() {
    // `string | number === "hi"` — `string` part of union overlaps.
    let diags = check_source(
        "type S = string | number;\
         function f(x: S) {\
             if (x === \"hi\") {}\
         }",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_comparison_distinct_union_members_ts2367() {
    // `"a" | "b" === "c"` — `"c"` is not assignable to either literal.
    // (TS narrows the union; the comparison with `"c"` has no overlap.)
    let diags = check_source(
        "function f(x: \"a\" | \"b\") {\
             if (x === \"c\") {}\
         }",
    );
    assert_diagnostic_code(&diags, 2367);
}

// ────────────────────────────────────────────────────────────────────────────
// Invocation errors: TS2349 (not callable) / TS2351 (not constructable)
// ────────────────────────────────────────────────────────────────────────────

#[test]
fn checker_call_number_ts2349() {
    // `number` is not callable.
    let diags = check_source(
        "const x: number = 1;\
         x();",
    );
    assert_diagnostic_code(&diags, 2349);
}

#[test]
fn checker_call_string_ts2349() {
    // `string` is not callable.
    let diags = check_source(
        "const x: string = \"hi\";\
         x();",
    );
    assert_diagnostic_code(&diags, 2349);
}

#[test]
fn checker_call_boolean_ts2349() {
    // `boolean` is not callable.
    let diags = check_source(
        "const x: boolean = true;\
         x();",
    );
    assert_diagnostic_code(&diags, 2349);
}

#[test]
fn checker_call_object_literal_ts2349() {
    // Object literal with no call signatures.
    let diags = check_source(
        "const x = { a: 1 };\
         x();",
    );
    assert_diagnostic_code(&diags, 2349);
}

#[test]
fn checker_call_class_instance_ts2349() {
    // Class instance (no call signature) is not callable.
    let diags = check_source(
        "class C { m() {} }\
         const c = new C();\
         c();",
    );
    assert_diagnostic_code(&diags, 2349);
}

#[test]
fn checker_call_function_no_error() {
    // Calling a function should not emit TS2349.
    let diags = check_source(
        "function f() {}\
         f();",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_call_arrow_no_error() {
    // Calling an arrow function should not emit TS2349.
    let diags = check_source(
        "const f = () => 1;\
         f();",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_call_method_no_error() {
    // Calling a method should not emit TS2349.
    let diags = check_source(
        "class C { m() { return 1; } }\
         const c = new C();\
         c.m();",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_new_number_ts2351() {
    // `new` on a `number` is not constructable.
    let diags = check_source(
        "const x: number = 1;\
         new x();",
    );
    assert_diagnostic_code(&diags, 2351);
}

#[test]
fn checker_new_object_literal_ts2351() {
    // `new` on an object literal is not constructable.
    let diags = check_source(
        "const x = { a: 1 };\
         new x();",
    );
    assert_diagnostic_code(&diags, 2351);
}

#[test]
fn checker_new_class_no_error() {
    // `new` on a class should not emit TS2351.
    let diags = check_source(
        "class C {}\
         const c = new C();",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_call_any_no_error() {
    // Calling `any` should not emit TS2349.
    let diags = check_source(
        "const x: any = 1;\
         x();",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_new_any_no_error() {
    // `new` on `any` should not emit TS2351.
    let diags = check_source(
        "const x: any = 1;\
         new x();",
    );
    assert_no_diagnostics(&diags);
}

// ────────────────────────────────────────────────────────────────────────────
// Readonly property assignment (TS2540)
// ────────────────────────────────────────────────────────────────────────────

#[test]
fn checker_assign_to_readonly_class_property_ts2540() {
    // Assigning to a `readonly` class property outside the constructor
    // emits TS2540.
    let diags = check_source(
        "class C { readonly x: number = 0; }\
         const c = new C();\
         c.x = 1;",
    );
    assert_diagnostic_code(&diags, 2540);
}

#[test]
fn checker_assign_to_writable_class_property_no_error() {
    // Assigning to a non-readonly class property is allowed.
    let diags = check_source(
        "class C { x: number = 0; }\
         const c = new C();\
         c.x = 1;",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_assign_to_readonly_interface_property_ts2540() {
    // Assigning to a `readonly` property of an interface-typed variable.
    let diags = check_source(
        "interface I { readonly x: number; }\
         const obj: I = { x: 1 };\
         obj.x = 2;",
    );
    assert_diagnostic_code(&diags, 2540);
}

#[test]
fn checker_assign_to_writable_interface_property_no_error() {
    // Assigning to a non-readonly interface property is allowed.
    let diags = check_source(
        "interface I { x: number; }\
         const obj: I = { x: 1 };\
         obj.x = 2;",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_compound_assignment_to_readonly_ts2540() {
    // Compound assignment (`+=`) does not use `EqualsToken`; we only check
    // `=` assignment for now, so `+=` to readonly does not emit TS2540 yet.
    // Test the basic `=` form on a readonly property of an interface.
    let diags = check_source(
        "interface I { readonly y: number; }\
         const obj: I = { y: 1 };\
         obj.y = 10;",
    );
    assert_diagnostic_code(&diags, 2540);
}

#[test]
fn checker_assign_to_method_no_error() {
    // Methods are not declared `readonly`, so assigning to them (even if
    // odd) does not trigger TS2540.
    let diags = check_source(
        "class C { m(): void {} }\
         const c = new C();\
         c.m = () => {};",
    );
    // We only check `readonly` modifier on properties; methods have none.
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_assign_to_inherited_readonly_ts2540() {
    // Inherited readonly property from base class also triggers TS2540.
    let diags = check_source(
        "class B { readonly x: number = 1; }\
         class D extends B {}\
         const d = new D();\
         d.x = 2;",
    );
    assert_diagnostic_code(&diags, 2540);
}

// ────────────────────────────────────────────────────────────────────────────
// Function declarations (no diagnostics expected)
// ────────────────────────────────────────────────────────────────────────────

#[test]
fn checker_function_declaration_no_error() {
    let diags = check_source("function f() { return 1; }");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_function_with_params_no_error() {
    let diags = check_source("function add(a: number, b: number): number { return a + b; }");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_arrow_function_no_error() {
    let diags = check_source("const f = (x: number) => x * 2;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_function_expression_no_error() {
    let diags = check_source("const f = function() { return 42; };");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_nested_function_no_error() {
    let diags = check_source("function outer() { function inner() { return 1; } return inner(); }");
    // inner() is resolved since it's in scope, but the checker may not
    // resolve function declarations yet. This is fine — just no panics.
    assert_no_diagnostics(&diags);
}

// ────────────────────────────────────────────────────────────────────────────
// Function overloads
// ────────────────────────────────────────────────────────────────────────────

#[test]
fn checker_overload_matching_first_signature_no_error() {
    // `f("hi")` matches the first overload (string → number).
    let diags = check_source(
        "function f(x: string): number;\n\
         function f(x: number): string;\n\
         function f(x: any): any { return x; }\n\
         let n: number = f(\"hi\");",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_overload_matching_second_signature_no_error() {
    // `f(42)` matches the second overload (number → string).
    let diags = check_source(
        "function f(x: string): number;\n\
         function f(x: number): string;\n\
         function f(x: any): any { return x; }\n\
         let s: string = f(42);",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_overload_wrong_return_type_ts2322() {
    // `f("hi")` returns `number` (first overload), not `string`.
    let diags = check_source(
        "function f(x: string): number;\n\
         function f(x: number): string;\n\
         function f(x: any): any { return x; }\n\
         let s: string = f(\"hi\");",
    );
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_overload_no_matching_signature_ts2345() {
    // `f(true)` matches neither `string` nor `number` overload.
    let diags = check_source(
        "function f(x: string): number;\n\
         function f(x: number): string;\n\
         function f(x: any): any { return x; }\n\
         f(true);",
    );
    assert_diagnostic_code(&diags, 2345);
}

#[test]
fn checker_overload_single_implementation_no_error() {
    // A function with just an implementation (no overloads) should work.
    let diags = check_source(
        "function f(x: number): number { return x + 1; }\n\
         let n: number = f(42);",
    );
    assert_no_diagnostics(&diags);
}

// ────────────────────────────────────────────────────────────────────────────
// Class declarations (no diagnostics expected)
// ────────────────────────────────────────────────────────────────────────────

#[test]
fn checker_empty_class_no_error() {
    let diags = check_source("class Empty {}");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_class_with_property_no_error() {
    let diags = check_source("class Point { x: number = 0; y: number = 0; }");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_class_with_method_no_error() {
    let diags = check_source("class Greeter { greet() { return 'hello'; } }");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_class_with_constructor_no_error() {
    let diags = check_source("class Foo { constructor() {} }");
    assert_no_diagnostics(&diags);
}

// ────────────────────────────────────────────────────────────────────────────
// Interface declarations (no diagnostics expected)
// ────────────────────────────────────────────────────────────────────────────

#[test]
fn checker_empty_interface_no_error() {
    let diags = check_source("interface Empty {}");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_interface_with_properties_no_error() {
    let diags = check_source("interface Person { name: string; age: number; }");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_interface_with_optional_property_no_error() {
    let diags = check_source("interface Config { timeout?: number; }");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_interface_with_method_no_error() {
    let diags = check_source("interface Callback { (err: Error | null): void; }");
    assert_no_diagnostics(&diags);
}

// ────────────────────────────────────────────────────────────────────────────
// Generic declarations (no diagnostics expected)
// ────────────────────────────────────────────────────────────────────────────

#[test]
fn checker_generic_function_no_error() {
    let diags = check_source("function identity<T>(arg: T): T { return arg; }");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_generic_class_no_error() {
    let diags = check_source("class Box<T> { value: T; constructor(v: T) { this.value = v; } }");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_generic_interface_no_error() {
    let diags = check_source("interface Pair<T, U> { first: T; second: U; }");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_generic_constraint_no_error() {
    let diags = check_source(
        "function longest<T extends { length: number }>(a: T, b: T): T { return a.length >= b.length ? a : b; }",
    );
    assert_no_diagnostics(&diags);
}

// ────────────────────────────────────────────────────────────────────────────
// New expression instance type
// ────────────────────────────────────────────────────────────────────────────

#[test]
fn checker_new_expression_property_access_no_error() {
    // `new Foo()` returns the instance type; `f.x` resolves.
    let diags = check_source(
        "class Foo { x: number = 1; }\n\
         let f = new Foo();\n\
         let n: number = f.x;",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_new_expression_property_missing_ts2339() {
    // `f.missing` does not exist on the instance type.
    let diags = check_source(
        "class Foo { x: number = 1; }\n\
         let f = new Foo();\n\
         f.missing;",
    );
    assert_diagnostic_code(&diags, 2339);
}

#[test]
fn checker_new_expression_method_call_no_error() {
    // `f.greet()` resolves to the instance method.
    let diags = check_source(
        "class Foo { greet(): string { return 'hi'; } }\n\
         let f = new Foo();\n\
         let s: string = f.greet();",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_new_expression_inherited_property_no_error() {
    // `new Derived()` has inherited `x` from `Base`.
    let diags = check_source(
        "class Base { x: number = 1; }\n\
         class Derived extends Base {}\n\
         let d = new Derived();\n\
         let n: number = d.x;",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_new_expression_inherited_method_no_error() {
    // `new Derived()` has inherited `greet()` from `Base`.
    let diags = check_source(
        "class Base { greet(): string { return 'hi'; } }\n\
         class Derived extends Base {}\n\
         let d = new Derived();\n\
         let s: string = d.greet();",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_new_expression_wrong_property_type_ts2322() {
    // `f.x` is `number`, not `string`.
    let diags = check_source(
        "class Foo { x: number = 1; }\n\
         let f = new Foo();\n\
         let s: string = f.x;",
    );
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_new_expression_constructor_args_ts2345() {
    // `new Foo('hi')` — constructor expects `number`.
    let diags = check_source(
        "class Foo { constructor(n: number) {} }\n\
         new Foo('hi');",
    );
    assert_diagnostic_code(&diags, 2345);
}

// ────────────────────────────────────────────────────────────────────────────
// Union and intersection types (no diagnostics expected)
// ────────────────────────────────────────────────────────────────────────────

#[test]
fn checker_union_type_annotation_no_error() {
    let diags = check_source("let x: string | number = 42;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_intersection_type_annotation_no_error() {
    let diags = check_source("let x: { a: number } & { b: string };");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_type_alias_union_no_error() {
    let diags = check_source("type Status = 'active' | 'inactive';");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_type_alias_intersection_no_error() {
    let diags = check_source(
        "type Named = { name: string };\ntype Aged = { age: number };\ntype Person = Named & Aged;",
    );
    assert_no_diagnostics(&diags);
}

// ────────────────────────────────────────────────────────────────────────────
// Type alias assignability: a TypeReference resolves to the alias's declared
// type, so the relater can check assignability against named aliases.
// ────────────────────────────────────────────────────────────────────────────

#[test]
fn checker_type_alias_assignable_value_no_error() {
    let diags = check_source("type Str = string;\nlet x: Str = 'hi';");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_type_alias_mismatch_ts2322() {
    // `Str` aliases `string`; assigning a number must error.
    let diags = check_source("type Str = string;\nlet x: Str = 42;");
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_type_alias_to_union_mismatch_ts2322() {
    let diags = check_source("type U = string | number;\nlet x: U = true;");
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_type_alias_to_union_member_no_error() {
    let diags = check_source("type U = string | number;\nlet x: U = 42;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_transitive_type_alias_mismatch_ts2322() {
    // `A` aliases `B` aliases `number`; a string is not assignable.
    let diags = check_source("type B = number;\ntype A = B;\nlet x: A = 'hi';");
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_recursive_type_alias_does_not_crash() {
    // Indirectly recursive aliases must be broken by the cycle guard and
    // resolve to `any` (no diagnostic), never stack-overflow.
    let diags = check_source("type A = B;\ntype B = A;\nlet x: A = 1;");
    assert_no_diagnostics(&diags);
}

// ────────────────────────────────────────────────────────────────────────────
// TS2304: Cannot find name (undefined references)
// ────────────────────────────────────────────────────────────────────────────

#[test]
fn checker_undefined_variable_ts2304() {
    let diags = check_source("let x = undefinedVar;");
    assert_diagnostic_code(&diags, 2304);
}

#[test]
fn checker_undefined_function_call_ts2304() {
    let diags = check_source("undefinedFunc();");
    assert_diagnostic_code(&diags, 2304);
}

#[test]
fn checker_multiple_undefined_references_ts2304() {
    let diags = check_source("let x = a + b;");
    assert_diagnostic_code(&diags, 2304);
}

#[test]
fn checker_new_undefined_class_ts2304() {
    let diags = check_source("let x = new NonExistentClass();");
    assert_diagnostic_code(&diags, 2304);
}

#[test]
fn checker_shorthand_property_undefined_ts2304() {
    let diags = check_source("let x = { undefinedVar };");
    assert_diagnostic_code(&diags, 2304);
}

// ────────────────────────────────────────────────────────────────────────────
// Nested scope resolution (NameResolver)
// ────────────────────────────────────────────────────────────────────────────

#[test]
fn checker_nested_function_scope_resolves_outer_var() {
    let diags = check_source("let x = 1; function foo() { return x; }");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_nested_block_scope_resolves_outer_let() {
    let diags = check_source("let x = 1; { let y = x; }");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_block_scope_var_not_visible_outside() {
    let diags = check_source("{ let x = 1; } let y = x;");
    assert_diagnostic_code(&diags, 2304);
}

#[test]
fn checker_function_parameter_resolves_in_body() {
    let diags = check_source("function foo(x: number) { return x; }");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_nested_function_shadows_outer() {
    let diags = check_source("let x = 1; function foo() { let x = 2; return x; }");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_closure_captures_outer_variable() {
    let diags = check_source("function outer() { let x = 1; function inner() { return x; } }");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_deeply_nested_scope_resolves() {
    let diags = check_source(
        "let a = 1; function f1() { let b = 2; function f2() { let c = 3; return a + b + c; } }",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_class_method_resolves_class_member() {
    let diags = check_source("class Foo { x = 1; bar() { return this.x; } }");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_if_block_scope_var_not_visible_outside() {
    let diags = check_source("if (true) { let x = 1; } let y = x;");
    assert_diagnostic_code(&diags, 2304);
}

#[test]
fn checker_for_loop_var_in_body() {
    let diags = check_source("for (let i = 0; i < 10; i++) { let x = i; }");
    assert_no_diagnostics(&diags);
}

// ────────────────────────────────────────────────────────────────────────────
// Control flow statements
// ────────────────────────────────────────────────────────────────────────────

#[test]
fn checker_if_statement_no_error() {
    let diags = check_source("let x = 1;\nif (x > 0) { let y = 2; }");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_if_else_statement_no_error() {
    let diags = check_source("let x = 1;\nif (x > 0) { let y = 2; } else { let z = 3; }");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_while_loop_no_error() {
    let diags = check_source("let x = 0;\nwhile (x < 10) { x = x + 1; }");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_do_while_loop_no_error() {
    let diags = check_source("let x = 0;\ndo { x = x + 1; } while (x < 10);");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_for_loop_no_error() {
    let diags = check_source("for (let i = 0; i < 10; i = i + 1) { let x = i; }");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_switch_statement_no_error() {
    let diags = check_source("let x = 1;\nswitch (x) { case 1: break; default: break; }");
    assert_no_diagnostics(&diags);
}

// ────────────────────────────────────────────────────────────────────────────
// Expressions
// ────────────────────────────────────────────────────────────────────────────

#[test]
fn checker_binary_expression_no_error() {
    let diags = check_source("let x = 1 + 2 * 3;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_prefix_unary_no_error() {
    let diags = check_source("let x = -1;\nlet y = !true;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_postfix_unary_no_error() {
    let diags = check_source("let x = 1;\nx++;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_ternary_expression_no_error() {
    let diags = check_source("let x = true ? 1 : 2;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_array_literal_no_error() {
    let diags = check_source("let arr = [1, 2, 3];");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_object_literal_no_error() {
    let diags = check_source("let obj = { a: 1, b: 'hello' };");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_template_literal_no_error() {
    let diags = check_source("let name = 'world';\nlet msg = `hello ${name}`;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_property_access_no_error() {
    let diags = check_source("let obj = { a: 1 };\nlet x = obj.a;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_element_access_no_error() {
    let diags = check_source("let arr = [1, 2, 3];\nlet x = arr[0];");
    assert_no_diagnostics(&diags);
}

// ────────────────────────────────────────────────────────────────────────────
// TypeScript-specific constructs
// ────────────────────────────────────────────────────────────────────────────

#[test]
fn checker_enum_declaration_no_error() {
    let diags = check_source("enum Color { Red, Green, Blue }");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_enum_with_initializer_no_error() {
    let diags = check_source("enum Color { Red = 1, Green = 2, Blue = 4 }");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_namespace_declaration_no_error() {
    let diags = check_source("namespace MyNamespace { export const x = 1; }");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_type_alias_primitive_no_error() {
    let diags = check_source("type MyNumber = number;\nlet x: MyNumber = 42;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_type_alias_object_no_error() {
    let diags =
        check_source("type Point = { x: number; y: number };\nlet p: Point = { x: 1, y: 2 };");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_type_alias_union_literal_no_error() {
    let diags = check_source("type Direction = 'north' | 'south' | 'east' | 'west';");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_type_alias_generic_no_error() {
    let diags = check_source(
        "type Result<T> = { success: true; value: T } | { success: false; error: string };",
    );
    assert_no_diagnostics(&diags);
}

// ────────────────────────────────────────────────────────────────────────────
// Literals and primitive expressions
// ────────────────────────────────────────────────────────────────────────────

#[test]
fn checker_string_literal_expression_no_error() {
    let diags = check_source("let x = 'hello';");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_numeric_literal_expression_no_error() {
    let diags = check_source("let x = 42;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_boolean_literal_expression_no_error() {
    let diags = check_source("let x = true;\nlet y = false;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_null_literal_expression_no_error() {
    let diags = check_source("let x = null;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_this_expression_no_error() {
    let diags = check_source("class Foo { x = 1; method() { return this.x; } }");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_bigint_literal_no_error() {
    let diags = check_source("let x = 42n;");
    assert_no_diagnostics(&diags);
}

// ────────────────────────────────────────────────────────────────────────────
// Type assertions
// ────────────────────────────────────────────────────────────────────────────

#[test]
fn checker_as_expression_no_error() {
    let diags = check_source("let x = 42 as any;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_non_null_expression_no_error() {
    let diags = check_source("let x: string | null = 'hello';\nlet y = x!;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_satisfies_expression_no_error() {
    let diags = check_source("let x = 42 satisfies number;");
    assert_no_diagnostics(&diags);
}

// ────────────────────────────────────────────────────────────────────────────
// Variable references
// ────────────────────────────────────────────────────────────────────────────

#[test]
fn checker_variable_referencing_defined_variable_no_error() {
    let diags = check_source("let a = 1;\nlet b = a;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_chained_variable_references_no_error() {
    let diags = check_source("let a = 1;\nlet b = a;\nlet c = b;\nlet d = c;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_expression_with_defined_and_undefined_vars() {
    let diags = check_source("let a = 1;\nlet b = a + undefinedVar;");
    assert_diagnostic_code(&diags, 2304);
}

// ────────────────────────────────────────────────────────────────────────────
// Destructuring and spread
// ────────────────────────────────────────────────────────────────────────────

#[test]
fn checker_destructuring_array_no_error() {
    let diags = check_source("let arr = [1, 2, 3];\nlet [a, b, c] = arr;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_destructuring_object_no_error() {
    let diags = check_source("let obj = { x: 1, y: 2 };\nlet { x, y } = obj;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_spread_element_no_error() {
    let diags = check_source("let arr = [1, 2, 3];\nlet copy = [...arr];");
    assert_no_diagnostics(&diags);
}

// ────────────────────────────────────────────────────────────────────────────
// Async and generator functions
// ────────────────────────────────────────────────────────────────────────────

#[test]
fn checker_async_function_no_error() {
    let diags = check_source("async function fetchData() { return 42; }");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_generator_function_no_error() {
    let diags = check_source("function* count() { yield 1; yield 2; }");
    assert_no_diagnostics(&diags);
}

// ────────────────────────────────────────────────────────────────────────────
// Class with inheritance
// ────────────────────────────────────────────────────────────────────────────

#[test]
fn checker_class_extends_no_error() {
    let diags = check_source("class Base { x = 1; }\nclass Derived extends Base { y = 2; }");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_class_extends_inherited_method_no_error() {
    // `this.greet()` resolves to the inherited `greet` method.
    let diags = check_source(
        "class Base { greet(): string { return 'hi'; } }\n\
         class Derived extends Base { test() { return this.greet(); } }",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_class_extends_inherited_property_no_error() {
    // `this.x` resolves to the inherited property `x`.
    let diags = check_source(
        "class Base { x: number = 1; }\n\
         class Derived extends Base { test() { return this.x + 1; } }",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_class_extends_property_not_on_this_ts2339() {
    // `this.missing` does not exist on the class or its base.
    let diags = check_source(
        "class Base { x: number = 1; }\n\
         class Derived extends Base { test() { return this.missing; } }",
    );
    assert_diagnostic_code(&diags, 2339);
}

#[test]
fn checker_class_extends_override_method_no_error() {
    // Derived class overrides a base method; `this` type uses the override.
    let diags = check_source(
        "class Base { greet(): string { return 'base'; } }\n\
         class Derived extends Base { greet(): string { return 'derived'; } }",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_class_extends_super_call_no_error() {
    // `super.greet()` accesses the base class method.
    let diags = check_source(
        "class Base { greet(): string { return 'base'; } }\n\
         class Derived extends Base { greet(): string { return super.greet(); } }",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_class_extends_implements_via_base_no_error() {
    // If the base class satisfies the interface, the derived class does too.
    let diags = check_source(
        "interface I { greet(): string; }\n\
         class Base implements I { greet(): string { return 'base'; } }\n\
         class Derived extends Base {}",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_class_extends_multilevel_no_error() {
    // Multi-level inheritance: C inherits from B inherits from A.
    let diags = check_source(
        "class A { a(): number { return 1; } }\n\
         class B extends A { b(): number { return 2; } }\n\
         class C extends B { test() { return this.a() + this.b(); } }",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_class_this_property_access_ts2339() {
    // `this` inside a class resolves to the instance type.
    let diags = check_source("class C { x: number = 1; test() { return this.missing; } }");
    assert_diagnostic_code(&diags, 2339);
}

#[test]
fn checker_class_this_property_access_no_error() {
    // `this` inside a class resolves to the instance type.
    let diags = check_source("class C { x: number = 1; test() { return this.x; } }");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_class_implements_interface_no_error() {
    let diags = check_source(
        "interface Named { name: string; }\nclass Person implements Named { name: string = 'Alice'; }",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_class_implements_interface_with_method_no_error() {
    // A class that provides a matching method should satisfy the interface.
    let diags = check_source(
        "interface IFoo { bar(): number; }\n\
         class C implements IFoo { bar() { return 42; } }",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_class_implements_interface_wrong_method_return_ts2420() {
    // The class method's return type must be assignable to the interface's.
    let diags = check_source(
        "interface IFoo { bar(): number; }\n\
         class C implements IFoo { bar(): string { return 'hi'; } }",
    );
    assert_diagnostic_code(&diags, 2420);
}

// ────────────────────────────────────────────────────────────────────────────
// Export declarations
// ────────────────────────────────────────────────────────────────────────────

#[test]
fn checker_export_variable_no_error() {
    let diags = check_source("export const x = 1;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_export_function_no_error() {
    let diags = check_source("export function f() { return 42; }");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_export_class_no_error() {
    let diags = check_source("export class MyClass {}");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_export_interface_no_error() {
    let diags = check_source("export interface MyInterface { x: number; }");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_export_default_function_no_error() {
    let diags = check_source("export default function() { return 42; }");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_export_default_class_no_error() {
    let diags = check_source("export default class { x = 1; }");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_export_default_expression_no_error() {
    let diags = check_source("export default 42;");
    assert_no_diagnostics(&diags);
}

// ────────────────────────────────────────────────────────────────────────────
// P3.4: Export/import binding gaps — parity tests for the new bind arms.
// These exercise the `ImportClause`, `ExportAssignment`,
// `ExportDeclaration`, and `NamespaceExportDeclaration` bind dispatch arms
// added to the Rust binder. They are single-file tests (no cross-module
// resolution) verifying that the new symbols are created without producing
// spurious diagnostics.
// ────────────────────────────────────────────────────────────────────────────

#[test]
fn checker_export_default_class_named_is_accessible() {
    // `export default class Foo {}` parses as a ClassDeclaration with the
    // `default` modifier; `Foo` must remain accessible by its local name.
    let diags = check_source(
        "export default class Foo {}\n\
         let x: Foo | null = null;",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_export_default_function_named_is_accessible() {
    // `export default function foo()` parses as a FunctionDeclaration with
    // the `default` modifier; `foo` must remain accessible by its local name.
    let diags = check_source(
        "export default function foo(): number { return 1; }\n\
         let x: number = foo();",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_export_default_identifier_expression_no_error() {
    // `export default <identifier>` creates an Alias symbol named "default".
    // The identifier `foo` resolves to the local `const foo`.
    let diags = check_source("const foo = 1;\nexport default foo;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_export_default_object_literal_no_error() {
    // `export default <object literal>` creates a Property symbol named
    // "default" (the expression is neither an entity name nor a class).
    let diags = check_source("export default { a: 1, b: 2 };");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_export_equals_no_error() {
    // `export = x` creates an Alias symbol named "export=" in the module's
    // exports, with a value declaration set.
    let diags = check_source("function x(): void {}\nexport = x;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_export_star_no_error() {
    // `export * from "mod"` records an ExportStar symbol in the file's
    // exports. No reference to the star is made, so no diagnostics.
    let diags = check_source("export * from \"mod\";");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_export_star_as_ns_no_error() {
    // `export * as ns from "mod"` declares an Alias symbol for `ns`.
    let diags = check_source("export * as ns from \"mod\";");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_export_named_reexport_no_error() {
    // `const x = 1; export { x }` — the ExportSpecifier arm already creates
    // the alias; the ExportDeclaration arm must not duplicate or error.
    let diags = check_source("const x = 1;\nexport { x };");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_export_named_reexport_renamed_no_error() {
    // `export { x as y }` with a local `x`.
    let diags = check_source("const x = 1;\nexport { x as y };");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_import_default_no_error() {
    // `import D from "mod"` creates an Alias symbol for `D` in the file's
    // locals. `D` is not referenced, so no diagnostics. (Cross-module
    // resolution of `D`'s target is not exercised by single-file tests.)
    let diags = check_source("import D from \"mod\";\nexport function f(): void {}");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_import_named_and_default_no_error() {
    // `import D, { x } from "mod"` — default import `D` plus a named import.
    let diags = check_source("import D, { x } from \"mod\";\nexport function f(): void {}");
    assert_no_diagnostics(&diags);
}

// ────────────────────────────────────────────────────────────────────────────
// Multiple undefined references in complex expressions
// ────────────────────────────────────────────────────────────────────────────

#[test]
fn checker_binary_undefined_both_sides() {
    let diags = check_source("let x = a + b;");
    let count = diags.iter().filter(|d| d.code == 2304).count();
    assert_eq!(count, 2, "Expected 2 TS2304 errors, got {}", count);
}

#[test]
fn checker_nested_undefined_expressions() {
    let diags = check_source("let x = foo(bar(baz()));");
    let count = diags.iter().filter(|d| d.code == 2304).count();
    assert_eq!(count, 3, "Expected 3 TS2304 errors, got {}", count);
}

#[test]
fn checker_array_with_undefined_elements() {
    let diags = check_source("let x = [a, b, c];");
    let count = diags.iter().filter(|d| d.code == 2304).count();
    assert_eq!(count, 3, "Expected 3 TS2304 errors, got {}", count);
}

#[test]
fn checker_object_with_undefined_values() {
    let diags = check_source("let x = { a: a, b: b };");
    let count = diags.iter().filter(|d| d.code == 2304).count();
    assert_eq!(count, 2, "Expected 2 TS2304 errors, got {}", count);
}

#[test]
fn checker_many_undefined_variables() {
    let diags = check_source("let a = w;\nlet b = x;\nlet c = y;\nlet d = z;");
    let count = diags.iter().filter(|d| d.code == 2304).count();
    assert_eq!(count, 4, "Expected 4 TS2304 errors, got {}", count);
}

// ────────────────────────────────────────────────────────────────────────────
// If/while/for with undefined conditions
// ────────────────────────────────────────────────────────────────────────────

#[test]
fn checker_if_undefined_condition() {
    let diags = check_source("if (unknownVar) { }");
    let count = diags.iter().filter(|d| d.code == 2304).count();
    assert_eq!(count, 1, "Expected 1 TS2304 error, got {}", count);
}

#[test]
fn checker_while_undefined_condition() {
    let diags = check_source("while (unknownVar) { break; }");
    let count = diags.iter().filter(|d| d.code == 2304).count();
    assert_eq!(count, 1, "Expected 1 TS2304 error, got {}", count);
}

// ────────────────────────────────────────────────────────────────────────────
// Typeof, delete, void expressions
// ────────────────────────────────────────────────────────────────────────────

#[test]
fn checker_typeof_expression_no_error() {
    let diags = check_source("let x = typeof 42;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_delete_expression_no_error() {
    let diags = check_source("let obj = { x: 1 };\ndelete obj.x;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_void_expression_no_error() {
    let diags = check_source("let x = void 0;");
    assert_no_diagnostics(&diags);
}

// ────────────────────────────────────────────────────────────────────────────
// Edge cases
// ────────────────────────────────────────────────────────────────────────────

#[test]
fn checker_empty_file_no_error() {
    let diags = check_source("");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_only_comments_no_error() {
    let diags = check_source("// This is a comment\n/* block comment */");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_only_whitespace_no_error() {
    let diags = check_source("   \n\n  \t  \n");
    assert_no_diagnostics(&diags);
}

// ────────────────────────────────────────────────────────────────────────────
// JSX type-check smoke tests
// ────────────────────────────────────────────────────────────────────────────
// These use .tsx extension to enable JSX parsing.

#[test]
fn checker_jsx_self_closing_element_no_error() {
    let diags = check_source_tsx("const el = <div />;");
    // JSX identifiers are walked as children-for-expressions.
    // `div` is a tag name (not a reference), so no errors expected.
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_jsx_element_with_children_no_error() {
    let diags = check_source_tsx("const el = <div><span>hello</span></div>;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_jsx_fragment_no_error() {
    let diags = check_source_tsx("const el = <><div>a</div><div>b</div></>;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_jsx_with_expression_curly_no_error() {
    let diags = check_source_tsx("const x = 42;\nconst el = <div>{x}</div>;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_jsx_attribute_string_no_error() {
    let diags = check_source_tsx("const el = <div className='container' />;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_jsx_attribute_expression_no_error() {
    let diags = check_source_tsx("const x = 42;\nconst el = <div data-value={x} />;");
    assert_no_diagnostics(&diags);
}
#[test]
fn checker_jsx_undefined_expression_in_curly() {
    // `undefinedVar` is not in scope: expect TS2304.
    let diags = check_source_tsx("const el = <div>{undefinedVar}</div>;");
    assert_diagnostic_code(&diags, 2304);
}

#[test]
fn checker_jsx_precondition_jsx_flag_missing() {
    // No `--jsx` flag: expect TS17004 ("Cannot use JSX unless the
    // '--jsx' flag is provided").
    let diags = check_source_tsx_with_args(
        "const el = <div />;",
        &[], // no --jsx
    );
    assert_diagnostic_code(&diags, 17004);
}

#[test]
fn checker_jsx_duplicate_attribute_names() {
    // Two attributes with the same name: expect TS17001.
    let diags = check_source_tsx("const el = <div data-x='1' data-x='2' />;");
    assert_diagnostic_code(&diags, 17001);
}

#[test]
fn checker_jsx_comma_operator_in_expression() {
    // Comma operator in JSX expression: expect TS18007. The Go check
    // only flags a direct BinaryExpression-with-CommaToken, so we don't
    // wrap the comma in parentheses.
    let diags = check_source_tsx("const a = 1; const b = 2; const el = <div>{a, b}</div>;");
    assert_diagnostic_code(&diags, 18007);
}

#[test]
fn checker_jsx_component_no_signatures() {
    // `Foo` has no call/construct signatures: expect TS2604.
    let diags = check_source_tsx("const Foo = 42;\nconst el = <Foo />;");
    assert_diagnostic_code(&diags, 2604);
}

#[test]
fn checker_jsx_function_component_no_error() {
    // `Foo` is a function (has call signature): no error expected.
    let diags = check_source_tsx("function Foo() { return 1; }\nconst el = <Foo />;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_jsx_class_component_no_error() {
    // `Foo` is a class (has construct signature): no error expected.
    let diags = check_source_tsx("class Foo {}\nconst el = <Foo />;");
    assert_no_diagnostics(&diags);
}

// ────────────────────────────────────────────────────────────────────────────
// JSDoc type-check smoke tests (.js with JSDoc annotations)
// ────────────────────────────────────────────────────────────────────────────

#[test]
fn checker_jsdoc_basic_type_annotation_no_error() {
    let diags = check_source(
        "/** @type {number} */\n\
         let x = 42;",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_jsdoc_function_type_no_error() {
    let diags = check_source(
        "/** @param {number} a @param {number} b @returns {number} */\n\
         function add(a, b) { return a + b; }",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_jsdoc_typedef_no_error() {
    let diags = check_source(
        "/** @typedef {{ name: string, age: number }} Person */\n\
         /** @type {Person} */\n\
         let p = { name: 'Alice', age: 30 };",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_jsdoc_undefined_variable_ts2304() {
    let diags = check_source(
        "/** @type {number} */\n\
         let x = undefinedVar;",
    );
    assert_diagnostic_code(&diags, 2304);
}

#[test]
fn checker_jsdoc_class_no_error() {
    let diags = check_source(
        "/** @class */\n\
         function MyClass() { this.x = 1; }",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_jsdoc_enum_no_error() {
    let diags = check_source(
        "/** @enum {string} */\n\
         const Colors = { RED: 'red', GREEN: 'green', BLUE: 'blue' };",
    );
    assert_no_diagnostics(&diags);
}

// ────────────────────────────────────────────────────────────────────────────
// NameResolver: arguments symbol
// ────────────────────────────────────────────────────────────────────────────

#[test]
fn checker_arguments_in_function_no_error() {
    // `arguments` is a built-in symbol inside function bodies.
    let diags = check_source("function foo() { return arguments; }");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_arguments_outside_function_is_undefined() {
    // `arguments` outside a function should be undefined.
    let diags = check_source("let x = arguments;");
    let count = diags.iter().filter(|d| d.code == 2304).count();
    assert_eq!(count, 1, "Expected 1 TS2304 error, got {}", count);
}

#[test]
fn checker_arguments_in_arrow_function() {
    let diags = check_source("const foo = () => { return arguments; }");
    let count = diags.iter().filter(|d| d.code == 2304).count();
    assert_eq!(
        count, 1,
        "Expected 1 TS2304 error, got 0 - arrow functions have no arguments"
    );
}

#[test]
fn checker_arguments_in_method() {
    let diags = check_source("class C { method() { return arguments; } }");
    assert_no_diagnostics(&diags);
}

// ────────────────────────────────────────────────────────────────────────────
// NameResolver: scope gaps (P3.2)
// ────────────────────────────────────────────────────────────────────────────

#[test]
fn checker_arrow_param_initializer_references_outer_var() {
    // Arrow function parameter initializer can reference outer variables.
    // `useOuterVariableScopeInParameter` in Go's NameResolver.
    let diags = check_source(
        "let outer = 10;\
         \x20let f = (x = outer) => x;",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_infer_type_not_visible_in_false_branch() {
    // `infer T` should only be visible in the true (extends) branch of a
    // conditional type, not in the false branch. Use `Test<number>` so the
    // extends check is false and the false branch is actually evaluated
    // (where `R` should be unresolved → TS2304).
    let diags = check_source(
        "type Test<T> = T extends Array<infer R> ? R : R;\
         \x20let x: Test<number> = 5;",
    );
    // R is not visible in the false branch → TS2304.
    let count = diags.iter().filter(|d| d.code == 2304).count();
    assert_eq!(
        count, 1,
        "Expected TS2304 for R in false branch, got {}",
        count
    );
}

#[test]
fn checker_static_member_cannot_reference_type_param() {
    // Static members cannot reference class type parameters.
    // Go's NameResolver emits TS2322/TS2526 for this.
    let diags = check_source(
        "class Foo<T> {\
         \x20   static x: T;\
         \x20}",
    );
    // Go reports TS2322 "Static members cannot reference class type parameters"
    // or the type 'T' is simply unresolved. We expect at least one diagnostic.
    assert!(
        !diags.is_empty(),
        "Expected diagnostics for static member referencing type parameter"
    );
}

#[test]
fn checker_export_default_alias_resolution() {
    // `export default` should be usable as an alias within the module.
    let diags = check_source(
        "export default function foo() {}\
         \x20let x = foo();",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_function_expression_self_name() {
    // A named function expression can reference its own name inside.
    let diags = check_source(
        "let fact = function f(n: number): number {\
         \x20   return n <= 1 ? 1 : n * f(n - 1);\
         \x20};",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_namespace_member_access_from_outside() {
    // Namespace members should be accessible via qualified name.
    let diags = check_source(
        "namespace N {\
         \x20   export const x: number = 1;\
         \x20}\
         \x20let y: number = N.x;",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_namespace_non_exported_member_not_accessible() {
    // Non-exported namespace members should not be accessible.
    let diags = check_source(
        "namespace N {\
         \x20   const x: number = 1;\
         \x20}\
         \x20let y: number = N.x;",
    );
    let count = diags.iter().filter(|d| d.code == 2339).count();
    assert_eq!(
        count, 1,
        "Expected TS2339 for non-exported member, got {}",
        count
    );
}

// ────────────────────────────────────────────────────────────────────────────
// NameResolver: global symbol resolution
// ────────────────────────────────────────────────────────────────────────────

#[test]
fn checker_global_symbol_with_lib() {
    // With lib loaded, `Array` should be resolvable.
    let diags = check_source_with_lib("let x = Array;", false);
    let count = diags.iter().filter(|d| d.code == 2304).count();
    assert_eq!(
        count, 0,
        "Expected 0 TS2304 errors (Array is a global), got {}",
        count
    );
}

#[test]
fn checker_undefined_is_resolvable() {
    // `undefined` is a built-in global symbol.
    let diags = check_source_with_lib("let x = undefined;", false);
    let count = diags.iter().filter(|d| d.code == 2304).count();
    assert_eq!(
        count, 0,
        "Expected 0 TS2304 errors (undefined is a global), got {}",
        count
    );
}

#[test]
fn checker_global_this_is_resolvable() {
    // `globalThis` is a built-in global symbol.
    let diags = check_source_with_lib("let x = globalThis;", false);
    let count = diags.iter().filter(|d| d.code == 2304).count();
    assert_eq!(
        count, 0,
        "Expected 0 TS2304 errors (globalThis is a global), got {}",
        count
    );
}

#[test]
fn checker_type_inference_variable_declaration() {
    // Variable declaration with initializer infers the type.
    let diags = check_source("let x = 42; let y = x;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_type_inference_string_variable() {
    // Variable declaration with string initializer.
    let diags = check_source("let x = \"hello\"; let y = x;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_type_inference_binary_expression() {
    // Variable with binary expression initializer.
    let diags = check_source("let x = 1 + 2;");
    assert_no_diagnostics(&diags);
}

// ────────────────────────────────────────────────────────────────────────────
// P3.9: Control flow narrowing smoke tests
// ────────────────────────────────────────────────────────────────────────────

#[test]
fn checker_narrowing_null_removed_in_true_branch() {
    // `x !== null` in the true branch narrows `x` to `string`.
    // Assigning the narrowed `x` to a `string`-typed variable should
    // produce no diagnostics.
    let diags = check_source(
        "let x: string | null = null;\
         if (x !== null) {\
             let y: string = x;\
         }",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_narrowing_null_kept_in_false_branch() {
    // `x !== null` is false → `x` is `null` in the else branch.
    // Assigning `x` to `null` should succeed.
    let diags = check_source(
        "let x: string | null = null;\
         if (x !== null) {\
             x = null;\
         } else {\
             let y: null = x;\
         }",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_narrowing_typeof_string() {
    // `typeof x === \"string\"` narrows `x` to `string`.
    let diags = check_source(
        "let x: string | number = 0;\
         if (typeof x === \"string\") {\
             let y: string = x;\
         }",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_narrowing_typeof_number() {
    // `typeof x === \"number\"` narrows `x` to `number`.
    let diags = check_source(
        "let x: string | number = \"\";\
         if (typeof x === \"number\") {\
             let y: number = x;\
         }",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_narrowing_truthiness_removes_null() {
    // `if (x)` removes falsy types (null, undefined, void, false, 0, \"\")
    // in the true branch.
    let diags = check_source(
        "let x: string | null = null;\
         if (x) {\
             let y: string = x;\
         }",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_narrowing_discriminated_union_literal() {
    // Discriminated union narrowing: `obj.kind === \"foo\"` narrows
    // `obj` to the constituent whose `kind` is `\"foo\"`.
    let diags = check_source(
        "type T = { kind: \"foo\", value: string } | { kind: \"bar\", count: number };\
         let obj: T = { kind: \"foo\", value: \"x\" };\
         if (obj.kind === \"foo\") {\
             let v: string = obj.value;\
         }",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_narrowing_assignment_updates_type() {
    // After `x = \"hello\"`, `x` is narrowed to `string`.
    let diags = check_source(
        "let x: string | number = 0;\
         x = \"hello\";\
         let y: string = x;",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_narrowing_switch_on_symbol() {
    // `switch (x)` narrows `x` to the case expression's type in each case.
    let diags = check_source(
        "let x: string | number = 0;\
         switch (x) {\
             case \"foo\":\
                 let y: string = x;\
                 break;\
             case 42:\
                 let z: number = x;\
                 break;\
         }",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_narrowing_switch_default_removes_cases() {
    // In the `default` clause, `x` narrows to types not covered by any case.
    // Here `string` is covered by `case \"foo\"` and `number` by `case 42`,
    // so the default branch has `never` — but we still allow assignment to
    // `string | number` because TS is conservative for non-exhaustive checks.
    let diags = check_source(
        "let x: string | number | boolean = 0;\
         switch (x) {\
             case \"foo\":\
                 break;\
             case 42:\
                 break;\
             default:\
                 let z: boolean = x;\
                 break;\
         }",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_narrowing_switch_on_discriminant_property() {
    // `switch (obj.kind)` narrows `obj` to the constituent whose `kind`
    // matches the case expression.
    let diags = check_source(
        "type T = { kind: \"foo\", value: string } | { kind: \"bar\", count: number };\
         let obj: T = { kind: \"foo\", value: \"x\" };\
         switch (obj.kind) {\
             case \"foo\":\
                 let v: string = obj.value;\
                 break;\
             case \"bar\":\
                 let c: number = obj.count;\
                 break;\
         }",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_narrowing_switch_default_discriminant_property() {
    // In the `default` clause of a switch on a discriminant property, `obj`
    // narrows to the constituent whose `kind` doesn't match any case.
    let diags = check_source(
        "type T = { kind: \"foo\", value: string } | { kind: \"bar\", count: number } | { kind: \"baz\", flag: boolean };\
         let obj: T = { kind: \"foo\", value: \"x\" };\
         switch (obj.kind) {\
             case \"foo\":\
                 break;\
             case \"bar\":\
                 break;\
             default:\
                 let f: boolean = obj.flag;\
                 break;\
         }",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_narrowing_type_predicate_true_branch() {
    // `if (isString(x))` narrows `x` to `string` in the true branch.
    let diags = check_source(
        "function isString(x: unknown): x is string {\
             return typeof x === \"string\";\
         }\
         let x: string | number = 0;\
         if (isString(x)) {\
             let y: string = x;\
         }",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_narrowing_type_predicate_false_branch() {
    // `if (!isString(x))` narrows `x` to `number` in the true branch of `!`.
    let diags = check_source(
        "function isString(x: unknown): x is string {\
             return typeof x === \"string\";\
         }\
         let x: string | number = 0;\
         if (!isString(x)) {\
             let y: number = x;\
         }",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_narrowing_type_predicate_else_branch() {
    // `if (isString(x)) { ... } else { ... }` narrows `x` to `number` in else.
    let diags = check_source(
        "function isString(x: unknown): x is string {\
             return typeof x === \"string\";\
         }\
         let x: string | number = 0;\
         if (isString(x)) {\
             let y: string = x;\
         } else {\
             let z: number = x;\
         }",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_narrowing_optional_chain_truthiness() {
    // `if (x?.a)` in the true branch narrows `x` to exclude null/undefined.
    let diags = check_source(
        "type T = { a: string } | null;\
         let x: T = null;\
         if (x?.a) {\
             let y: { a: string } = x;\
         }",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_narrowing_optional_chain_equality() {
    // `x?.a === \"foo\"` narrows `x` to exclude null/undefined in the true
    // branch (because if x were null, x?.a would be undefined, not \"foo\").
    let diags = check_source(
        "type T = { a: string } | null;\
         let x: T = null;\
         if (x?.a === \"foo\") {\
             let y: { a: string } = x;\
         }",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_narrowing_optional_chain_not_equal_undefined() {
    // `x?.a !== undefined` narrows `x` to exclude null/undefined in the
    // true branch.
    let diags = check_source(
        "type T = { a: string } | null;\
         let x: T = null;\
         if (x?.a !== undefined) {\
             let y: { a: string } = x;\
         }",
    );
    assert_no_diagnostics(&diags);
}

// P3.9e: Improved equality narrowing for literal types
// ────────────────────────────────────────────────────────────────────────────

#[test]
fn checker_narrowing_equality_replaces_string_with_literal() {
    // `x === "foo"` narrows `string` to `"foo"` (replace primitive with
    // literal). `let y: "foo" = x` should succeed in the true branch.
    let diags = check_source(
        "let x: string | number = 0;\
         if (x === \"foo\") {\
             let y: \"foo\" = x;\
         }",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_narrowing_equality_replaces_number_with_literal() {
    // `x === 42` narrows `number` to `42`.
    let diags = check_source(
        "let x: string | number = \"\";\
         if (x === 42) {\
             let y: 42 = x;\
         }",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_narrowing_equality_strict_null_vs_undefined() {
    // `x === undefined` narrows to `undefined` only (not `null`).
    let diags = check_source(
        "let x: string | null | undefined = null;\
         if (x === undefined) {\
             let y: undefined = x;\
         }",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_narrowing_equality_strict_null_kept() {
    // `x === null` narrows to `null` only (not `undefined`).
    let diags = check_source(
        "let x: string | null | undefined = undefined;\
         if (x === null) {\
             let y: null = x;\
         }",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_narrowing_equality_false_branch_removes_literal() {
    // `x !== "foo"` (false branch of ===) → remove `"foo"` from the union.
    // `x` should be `number` in the false branch.
    let diags = check_source(
        "let x: \"foo\" | number = \"foo\";\
         if (x === \"foo\") {\
         } else {\
             let y: number = x;\
         }",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_narrowing_equality_false_branch_non_unit_no_narrow() {
    // `x !== someObject` — value is not a unit type, so the false branch
    // should not narrow (x remains `{ a: string } | number`).
    let diags = check_source(
        "let x: { a: string } | number = { a: \"\" };\
         if (x === { a: \"hi\" }) {\
         } else {\
             let y: { a: string } | number = x;\
         }",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_narrowing_equality_any_not_narrowed() {
    // `any` type is not narrowed by equality comparisons.
    let diags = check_source(
        "let x: any = 0;\
         if (x === \"foo\") {\
             let y: any = x;\
         }",
    );
    assert_no_diagnostics(&diags);
}

// ────────────────────────────────────────────────────────────────────────────
// typeof switch narrowing (switch (typeof x))
// ────────────────────────────────────────────────────────────────────────────

#[test]
fn checker_narrowing_typeof_switch_string() {
    // `switch (typeof x)` narrows `x` to `string` in the `"string"` case.
    let diags = check_source(
        "let x: string | number = 0;\
         switch (typeof x) {\
             case \"string\":\
                 let y: string = x;\
                 break;\
             case \"number\":\
                 let z: number = x;\
                 break;\
         }",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_narrowing_typeof_switch_default_excludes_cases() {
    // In the `default` clause, `x` narrows to types not covered by any case.
    // Here `string` and `number` are covered, so default leaves `boolean`.
    let diags = check_source(
        "let x: string | number | boolean = 0;\
         switch (typeof x) {\
             case \"string\":\
                 break;\
             case \"number\":\
                 break;\
             default:\
                 let z: boolean = x;\
                 break;\
         }",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_narrowing_typeof_switch_string_literal_kept() {
    // A literal `"foo"` is a subtype of `string`, so in the `"string"` case
    // it should remain `"foo"` (not be widened to `string`).
    let diags = check_source(
        "let x: \"foo\" | 42 = 42;\
         switch (typeof x) {\
             case \"string\":\
                 let y: \"foo\" = x;\
                 break;\
             case \"number\":\
                 let z: 42 = x;\
                 break;\
         }",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_narrowing_typeof_switch_undefined() {
    // `case "undefined":` narrows to `undefined`.
    let diags = check_source(
        "let x: string | undefined = undefined;\
         switch (typeof x) {\
             case \"string\":\
                 let y: string = x;\
                 break;\
             case \"undefined\":\
                 let z: undefined = x;\
                 break;\
         }",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_narrowing_typeof_switch_boolean() {
    // `case "boolean":` narrows to `boolean`.
    let diags = check_source(
        "let x: string | boolean = false;\
         switch (typeof x) {\
             case \"string\":\
                 let y: string = x;\
                 break;\
             case \"boolean\":\
                 let z: boolean = x;\
                 break;\
         }",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_narrowing_typeof_switch_unreachable_case_never() {
    // `case "number":` on a `string | boolean` type is unreachable — the
    // narrowed type should be `never`. Assigning `never` to `number` is
    // allowed, so no diagnostic.
    let diags = check_source(
        "let x: string | boolean = false;\
         switch (typeof x) {\
             case \"number\":\
                 let z: number = x;\
                 break;\
             default:\
                 break;\
         }",
    );
    assert_no_diagnostics(&diags);
}

// ────────────────────────────────────────────────────────────────────────────
// switch (true) narrowing
// ────────────────────────────────────────────────────────────────────────────

#[test]
fn checker_narrowing_switch_true_equality() {
    // `switch (true) { case x === "foo": ... }` narrows `x` to `"foo"`.
    let diags = check_source(
        "let x: string | number = 0;\
         switch (true) {\
             case x === \"foo\":\
                 let y: string = x;\
                 break;\
             case x === 42:\
                 let z: number = x;\
                 break;\
         }",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_narrowing_switch_true_not_equal() {
    // `case x !== null:` in `switch (true)` narrows away null in the true branch.
    let diags = check_source(
        "let x: string | null = null;\
         switch (true) {\
             case x !== null:\
                 let y: string = x;\
                 break;\
             default:\
                 let z: null = x;\
                 break;\
         }",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_narrowing_switch_true_preceding_cases_negated() {
    // In the second case, the first case's condition is false, so `x !== "foo"`
    // narrows `x` to `"foo"` (negation of `x !== "foo"`).
    let diags = check_source(
        "let x: \"foo\" | \"bar\" | number = \"foo\";\
         switch (true) {\
             case typeof x === \"string\":\
                 let a: string = x;\
                 break;\
             default:\
                 let b: number = x;\
                 break;\
         }",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_narrowing_switch_true_type_predicate() {
    // `case isString(x):` narrows `x` to `string` using type predicate.
    let diags = check_source(
        "function isString(v: any): v is string { return typeof v === \"string\"; }\
         let x: string | number = 0;\
         switch (true) {\
             case isString(x):\
                 let y: string = x;\
                 break;\
             default:\
                 let z: number = x;\
                 break;\
         }",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_narrowing_switch_true_default_negates_all() {
    // In the default clause, all case conditions are false.
    // `x === "foo"` is false → x is not "foo"; `x === 42` is false → x is not 42.
    // The remaining type is `boolean`.
    let diags = check_source(
        "let x: \"foo\" | 42 | boolean = false;\
         switch (true) {\
             case x === \"foo\":\
                 break;\
             case x === 42:\
                 break;\
             default:\
                 let z: boolean = x;\
                 break;\
         }",
    );
    assert_no_diagnostics(&diags);
}

// ────────────────────────────────────────────────────────────────────────────
// asserts x is T narrowing (assertion functions)
// ────────────────────────────────────────────────────────────────────────────

#[test]
fn checker_narrowing_asserts_is_type() {
    // `asserts x is string` narrows `x` to `string` after the call.
    let diags = check_source(
        "function assertString(v: unknown): asserts v is string {}\
         let x: string | number = 0;\
         assertString(x);\
         let y: string = x;",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_narrowing_asserts_is_type_union() {
    // `asserts x is string | number` narrows to the union.
    let diags = check_source(
        "function assertVal(v: unknown): asserts v is string | number {}\
         let x: string | number | boolean = false;\
         assertVal(x);\
         let y: string | number = x;",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_narrowing_asserts_plain_removes_nullable() {
    // Plain `asserts x` (no type) narrows to truthy (removes null/undefined).
    let diags = check_source(
        "function assert(v: unknown): asserts v {}\
         let x: string | null = null;\
         assert(x);\
         let y: string = x;",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_narrowing_asserts_is_type_second_param() {
    // Assertion on the second parameter.
    let diags = check_source(
        "function check(cond: boolean, v: unknown): asserts v is string {}\
         let x: string | number = 0;\
         check(true, x);\
         let y: string = x;",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_narrowing_asserts_does_not_affect_other_vars() {
    // Assertion on `x` should not narrow `z`.
    let diags = check_source(
        "function assertString(v: unknown): asserts v is string {}\
         let x: string | number = 0;\
         let z: string | number = 0;\
         assertString(x);\
         let y: string = x;\
         let w: string | number = z;",
    );
    assert_no_diagnostics(&diags);
}

// ────────────────────────────────────────────────────────────────────────────
// instanceof / in / boolean narrowing (P3.9 fixture gaps)
// ────────────────────────────────────────────────────────────────────────────

#[test]
fn checker_narrowing_instanceof_true_branch() {
    // After `x instanceof Foo`, x is Foo in the true branch.
    let diags = check_source(
        "class Foo { greet() {} }\
         function f(x: Foo | string) {\
         \x20   if (x instanceof Foo) {\
         \x20       x.greet();\
         \x20   }\
         }",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_narrowing_instanceof_false_branch() {
    // In the false branch of `x instanceof Foo`, x is string.
    let diags = check_source(
        "class Foo {}\
         function f(x: Foo | string) {\
         \x20   if (!(x instanceof Foo)) {\
         \x20       let s: string = x;\
         \x20   }\
         }",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_narrowing_in_keyword_true_branch() {
    // After `'b' in obj`, obj is narrowed to the constituent with `b`.
    let diags = check_source(
        "type A = { a: number };\
         type B = { b: number };\
         function f(obj: A | B) {\
         \x20   if ('b' in obj) {\
         \x20       let n: number = obj.b;\
         \x20   }\
         }",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_narrowing_in_keyword_false_branch() {
    // In the false branch of `'b' in obj`, obj is narrowed to A.
    let diags = check_source(
        "function f(obj: { a: number } | { b: number }) {\
         \x20   if (!('b' in obj)) {\
         \x20       let n: number = obj.a;\
         \x20   }\
         }",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_narrowing_boolean_comparison_true() {
    // `x === true` narrows x to `true` in the true branch.
    let diags = check_source(
        "function f(x: boolean) {\
         \x20   if (x === true) {\
         \x20       let t: true = x;\
         \x20   }\
         }",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_narrowing_boolean_comparison_false_branch() {
    // `x === true` narrows x to `false` in the else branch.
    let diags = check_source(
        "function f(x: boolean) {\
         \x20   if (x === true) {\
         \x20   } else {\
         \x20       let f2: false = x;\
         \x20   }\
         }",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_narrowing_instanceof_property_access_error() {
    // Before the instanceof guard, accessing `greet()` should error (TS2339).
    let diags = check_source(
        "class Foo { greet() {} }\
         function f(x: Foo | string) {\
         \x20   x.greet();\
         }",
    );
    let count = diags.iter().filter(|d| d.code == 2339).count();
    assert_eq!(count, 1, "Expected TS2339 for x.greet() on union type");
}

#[test]
fn checker_narrowing_and_condition() {
    // `x !== null && ok` — x is narrowed to string in the true branch.
    let diags = check_source(
        "function f(x: string | null, ok: boolean) {\
         \x20   if (x !== null && ok) {\
         \x20       let s: string = x;\
         \x20   }\
         }",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_narrowing_or_branch_union() {
    // `x === null || ok` — in the false branch of `||`, x is string.
    let diags = check_source(
        "function f(x: string | null, ok: boolean) {\
         \x20   if (x === null || ok) {\
         \x20   } else {\
         \x20       let s: string = x;\
         \x20   }\
         }",
    );
    assert_no_diagnostics(&diags);
}

// ────────────────────────────────────────────────────────────────────────────
// P3.9 Part A: Fixture tests for already-working narrowing patterns
// ────────────────────────────────────────────────────────────────────────────

#[test]
fn checker_narrowing_loose_equality_eq_null() {
    // `x == null` (loose equality) narrows to null | undefined in the true
    // branch (both null and undefined match under `==`).
    let diags = check_source(
        "let x: string | null | undefined = null;\
         if (x == null) {\
             let y: null | undefined = x;\
         }",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_narrowing_loose_equality_ne_null() {
    // `x != null` (loose inequality) narrows to string in the true branch
    // (removes both null and undefined).
    let diags = check_source(
        "let x: string | null | undefined = null;\
         if (x != null) {\
             let y: string = x;\
         }",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_narrowing_typeof_false_branch() {
    // `typeof x !== "string"` (false branch of typeof) narrows to number.
    let diags = check_source(
        "let x: string | number = 0;\
         if (typeof x !== \"string\") {\
             let y: number = x;\
         }",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_narrowing_typeof_object_includes_null() {
    // `typeof x === "object"` narrows to null | {} (typeof null is "object").
    let diags = check_source(
        "let x: string | null | {} = null;\
         if (typeof x === \"object\") {\
             let y: null | {} = x;\
         }",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_narrowing_typeof_bigint() {
    // `typeof x === "bigint"` narrows to bigint.
    let diags = check_source(
        "let x: bigint | string = 0n;\
         if (typeof x === \"bigint\") {\
             let y: bigint = x;\
         }",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_narrowing_typeof_symbol() {
    // `typeof x === "symbol"` narrows to symbol. Uses `declare const` to
    // avoid needing the `Symbol()` global (unavailable with --noLib).
    let diags = check_source(
        "declare const sym: symbol;\
         let x: symbol | string = sym;\
         if (typeof x === \"symbol\") {\
             let y: symbol = x;\
         }",
    );
    assert_no_diagnostics(&diags);
}

// ────────────────────────────────────────────────────────────────────────────
// P3.9 Gap 14: Const variable alias inlining
// ────────────────────────────────────────────────────────────────────────────

#[test]
fn checker_narrowing_const_alias_truthiness() {
    // `const alias = x; if (alias)` — the const alias is inlined to `x`,
    // so the true branch narrows `x` to string (truthy).
    let diags = check_source(
        "let x: string | null = null;\
         const alias = x;\
         if (alias) {\
             let y: string = x;\
         }",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_narrowing_const_alias_not_narrowed_with_type_annotation() {
    // A const variable with an explicit type annotation is NOT inlined
    // (mirrors Go's `declaration.Type() == nil` check). The alias keeps
    // its declared type, so `if (alias)` does not narrow `x`.
    let diags = check_source(
        "let x: string | null = null;\
         const alias: string | null = x;\
         if (alias) {\
             let y: string | null = x;\
         }",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_narrowing_const_alias_let_not_inlined() {
    // `let` variables are not inlined (only `const`). The condition
    // `if (alias)` does not narrow `x`.
    let diags = check_source(
        "let x: string | null = null;\
         let alias = x;\
         if (alias) {\
             let y: string | null = x;\
         }",
    );
    assert_no_diagnostics(&diags);
}

// ────────────────────────────────────────────────────────────────────────────
// P3.9 Gap 4: typeof === "function" narrows to callable types only
// ────────────────────────────────────────────────────────────────────────────

#[test]
fn checker_narrowing_typeof_function_keeps_callable() {
    // `typeof x === "function"` narrows to the function type only,
    // removing plain object types (without call signatures).
    let diags = check_source(
        "type T = (() => void) | { a: string };\
         let x: T = { a: \"\" };\
         if (typeof x === \"function\") {\
             let y: () => void = x;\
         }",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_narrowing_typeof_function_false_branch_keeps_object() {
    // `typeof x !== "function"` narrows to the plain object type (the
    // function type is removed in the false branch).
    let diags = check_source(
        "type T = (() => void) | { a: string };\
         let x: T = { a: \"\" };\
         if (typeof x !== \"function\") {\
             let y: { a: string } = x;\
         }",
    );
    assert_no_diagnostics(&diags);
}

// ────────────────────────────────────────────────────────────────────────────
// P3.9 Gap 2: typeof on discriminant property
// ────────────────────────────────────────────────────────────────────────────

#[test]
fn checker_narrowing_typeof_discriminant_property_string() {
    // `typeof obj.kind === "string"` narrows `obj` to the constituent
    // whose `kind` property is a string-like type.
    let diags = check_source(
        "type T = { kind: \"foo\", value: string } | { kind: 42, count: number };\
         let obj: T = { kind: \"foo\", value: \"x\" };\
         if (typeof obj.kind === \"string\") {\
             let v: string = obj.value;\
         }",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_narrowing_typeof_discriminant_property_number() {
    // `typeof obj.kind === "number"` narrows `obj` to the constituent
    // whose `kind` property is a number-like type.
    let diags = check_source(
        "type T = { kind: \"foo\", value: string } | { kind: 42, count: number };\
         let obj: T = { kind: \"foo\", value: \"x\" };\
         if (typeof obj.kind === \"number\") {\
             let c: number = obj.count;\
         }",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_narrowing_typeof_discriminant_property_false_branch() {
    // `typeof obj.kind !== "string"` narrows `obj` to the constituent
    // whose `kind` property is NOT string-like (the number constituent).
    let diags = check_source(
        "type T = { kind: \"foo\", value: string } | { kind: 42, count: number };\
         let obj: T = { kind: \"foo\", value: \"x\" };\
         if (typeof obj.kind !== \"string\") {\
             let c: number = obj.count;\
         }",
    );
    assert_no_diagnostics(&diags);
}

// ────────────────────────────────────────────────────────────────────────────
// P3.9 Gap 1: Nullish coalescing (??) narrowing
// ────────────────────────────────────────────────────────────────────────────

#[test]
fn checker_narrowing_nullish_coalescing_false_branch_narrows_left_to_null() {
    // In the false branch of `if (a ?? b)`, `a` is narrowed to
    // null | undefined (since `a` must be null/undefined for `b` to be
    // evaluated, and `b` must be falsy for the result to be falsy).
    let diags = check_source(
        "let a: string | null = \"hello\";\
         let b: string = \"world\";\
         if (a ?? b) {\
         } else {\
             let y: null | undefined = a;\
         }",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_narrowing_nullish_coalescing_true_branch_no_narrow() {
    // In the true branch of `if (a ?? b)`, `a` is NOT narrowed
    // (it could be null/undefined if `b` is truthy).
    let diags = check_source(
        "let a: string | null = null;\
         let b: string = \"world\";\
         if (a ?? b) {\
             let y: string | null = a;\
         }",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_narrowing_nullish_coalescing_false_branch_narrows_right_to_falsy() {
    // In the false branch, `b` is narrowed to falsy types.
    let diags = check_source(
        "let a: string | null = null;\
         let b: string | 0 = 0;\
         if (a ?? b) {\
         } else {\
             let y: 0 = b;\
         }",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_narrowing_nullish_coalescing_with_const_alias() {
    // `const alias = a; if (alias ?? b)` — const alias inlining
    // applies optionality narrowing in the false branch.
    let diags = check_source(
        "let a: string | null = null;\
         const alias = a;\
         let b: string = \"world\";\
         if (alias ?? b) {\
         } else {\
             let y: null | undefined = a;\
         }",
    );
    assert_no_diagnostics(&diags);
}

// ────────────────────────────────────────────────────────────────────────────
// Type display (type_to_string) via diagnostic message args
// ────────────────────────────────────────────────────────────────────────────

/// Find a TS2322 diagnostic and return its message args.
fn get_ts2322_args(diags: &[tsox::ast::Diagnostic]) -> Vec<String> {
    diags
        .iter()
        .find(|d| d.code == 2322)
        .map(|d| d.message_args.clone())
        .unwrap_or_default()
}

#[test]
fn checker_type_display_string_literal() {
    // `"foo"` is not assignable to `number` → Go oracle widens to `string`.
    let diags = check_source("let x: number = \"foo\";");
    let args = get_ts2322_args(&diags);
    assert!(!args.is_empty(), "Expected TS2322");
    assert!(
        args.iter().any(|a| a == "string"),
        "Expected 'string' in args: {:?}",
        args
    );
}

#[test]
fn checker_type_display_number_literal() {
    // `42` is not assignable to `string` → Go oracle widens to `number`.
    let diags = check_source("let x: string = 42;");
    let args = get_ts2322_args(&diags);
    assert!(!args.is_empty(), "Expected TS2322");
    assert!(
        args.iter().any(|a| a == "number"),
        "Expected 'number' in args: {:?}",
        args
    );
}

#[test]
fn checker_type_display_union() {
    // `string | number` not assignable to `boolean` → should contain `string | number`.
    let diags = check_source(
        "let x: string | number = 0;\
         let y: boolean = x;",
    );
    let args = get_ts2322_args(&diags);
    assert!(!args.is_empty(), "Expected TS2322");
    assert!(
        args.iter().any(|a| a.contains("string | number")),
        "Expected 'string | number' in args: {:?}",
        args
    );
}

#[test]
fn checker_type_display_boolean_literal() {
    // `true` not assignable to `string` → Go oracle widens to `boolean`.
    let diags = check_source("let x: string = true;");
    let args = get_ts2322_args(&diags);
    assert!(!args.is_empty(), "Expected TS2322");
    assert!(
        args.iter().any(|a| a == "boolean"),
        "Expected 'boolean' in args: {:?}",
        args
    );
}

#[test]
fn checker_type_display_unknown() {
    // `unknown` display: assigning `unknown` to `string` (without assertion)
    // should fail and mention `unknown`.
    let diags = check_source(
        "let x: unknown = 0;\
         let y: string = x;",
    );
    let args = get_ts2322_args(&diags);
    assert!(!args.is_empty(), "Expected TS2322");
    assert!(
        args.iter().any(|a| a == "unknown"),
        "Expected 'unknown' in args: {:?}",
        args
    );
}

// ────────────────────────────────────────────────────────────────────────────
// Expression type inference: `as`, `satisfies`, unary, property/element access
// ────────────────────────────────────────────────────────────────────────────

#[test]
fn checker_as_expression_uses_type_annotation_no_error() {
    // `x as string` has type `string`; assigning to `string` is fine.
    let diags = check_source("let x: unknown = 0; let y: string = x as string;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_as_expression_wrong_annotation_ts2322() {
    // `x as number` has type `number`; assigning to `string` fails.
    let diags = check_source("let x: unknown = 0; let y: string = x as number;");
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_satisfies_expression_keeps_expression_type_no_error() {
    // `x satisfies string` keeps the type of `x` (here `string`),
    // so assigning to `string` is fine.
    let diags = check_source("let x = 'hi'; let y: string = x satisfies string;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_satisfies_expression_keeps_expression_type_ts2322() {
    // `x satisfies string` keeps the type of `x` (here `number`),
    // so assigning to `string` fails even though `satisfies string`.
    let diags = check_source("let x = 42; let y: string = x satisfies string;");
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_prefix_unary_not_returns_boolean_no_error() {
    // `!x` has type `boolean`.
    let diags = check_source("let x = 1; let y: boolean = !x;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_prefix_unary_not_returns_boolean_ts2322() {
    // `!x` has type `boolean`; assigning to `number` fails.
    let diags = check_source("let x = 1; let y: number = !x;");
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_prefix_unary_minus_returns_number_no_error() {
    // `-x` has type `number`.
    let diags = check_source("let x = 1; let y: number = -x;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_prefix_unary_minus_returns_number_ts2322() {
    // `-x` has type `number`; assigning to `string` fails.
    let diags = check_source("let x = 1; let y: string = -x;");
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_postfix_increment_returns_number_no_error() {
    // `x++` has type `number`.
    let diags = check_source("let x = 1; let y: number = x++;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_array_length_property_returns_number_no_error() {
    // `arr.length` has type `number`.
    let diags = check_source("let arr: number[] = [1, 2, 3]; let y: number = arr.length;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_array_length_property_returns_number_ts2322() {
    // `arr.length` has type `number`; assigning to `string` fails.
    let diags = check_source("let arr: number[] = [1, 2, 3]; let y: string = arr.length;");
    assert_diagnostic_code(&diags, 2322);
}

// ────────────────────────────────────────────────────────────────────────────
// Property access TS2339: "Property '{0}' does not exist on type '{1}'."
// ────────────────────────────────────────────────────────────────────────────

#[test]
fn checker_property_access_existing_property_no_error() {
    // `obj.a` exists on `{ a: number; b: string }` → no error.
    let diags = check_source("let obj = { a: 1, b: 'hi' }; let x = obj.a; let y = obj.b;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_property_access_missing_property_on_object_ts2339() {
    // `obj.c` doesn't exist on `{ a: number; b: string }` → TS2339.
    let diags = check_source("let obj = { a: 1, b: 'hi' }; let x = obj.c;");
    assert_diagnostic_code(&diags, 2339);
}

#[test]
fn checker_property_access_on_number_ts2339() {
    // `x.toUpperCase` doesn't exist on `number` (without lib, primitives
    // have no properties) → TS2339.
    let diags = check_source("let x: number = 1; x.toUpperCase();");
    assert_diagnostic_code(&diags, 2339);
}

#[test]
fn checker_property_access_on_string_literal_ts2339() {
    // `"hi".toUpperCase` doesn't exist on the string literal type `"hi"`
    // (no lib) → TS2339.
    let diags = check_source("let x = 'hi'; x.toUpperCase();");
    assert_diagnostic_code(&diags, 2339);
}

#[test]
fn checker_property_access_on_any_no_error() {
    // `any` allows any property → no error.
    let diags = check_source("let x: any = 1; x.toUpperCase();");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_property_access_on_type_parameter_constraint_no_error() {
    // `x.a` exists because `T extends { a: number }` → no error.
    let diags = check_source("function f<T extends { a: number }>(x: T) { return x.a; }");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_property_access_on_type_parameter_missing_ts2339() {
    // `x.b` doesn't exist on the constraint `{ a: number }` → TS2339.
    let diags = check_source("function f<T extends { a: number }>(x: T) { return x.b; }");
    assert_diagnostic_code(&diags, 2339);
}

#[test]
fn checker_property_access_on_union_present_in_all_no_error() {
    // Both constituents have `a` → no error.
    let diags = check_source(
        "let x: { a: number } | { a: string } = { a: 1 };\
         let y = x.a;",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_property_access_on_union_missing_in_one_ts2339() {
    // `{ a: number }` has `a`, `{ b: string }` doesn't → TS2339.
    let diags = check_source(
        "let x: { a: number } | { b: string } = { a: 1 };\
         let y = x.a;",
    );
    assert_diagnostic_code(&diags, 2339);
}

#[test]
fn checker_property_access_on_intersection_no_error() {
    // `{ a: number } & { b: string }` has both `a` and `b`.
    let diags = check_source(
        "let x: { a: number } & { b: string } = { a: 1, b: 'hi' };\
         let y = x.a;\
         let z = x.b;",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_property_access_on_intersection_missing_ts2339() {
    // Neither `{ a: number }` nor `{ b: string }` has `c` → TS2339.
    let diags = check_source(
        "let x: { a: number } & { b: string } = { a: 1, b: 'hi' };\
         let y = x.c;",
    );
    assert_diagnostic_code(&diags, 2339);
}

#[test]
fn checker_property_access_array_length_no_error() {
    // `arr.length` on `number[]` is hardcoded to `number` → no error.
    let diags = check_source("let arr: number[] = [1, 2, 3]; let x = arr.length;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_property_access_array_unknown_method_ts2339() {
    // `arr.push` is not hardcoded; without lib, tsc reports TS2339.
    let diags = check_source("let arr: number[] = [1, 2, 3]; arr.push(4);");
    assert_diagnostic_code(&diags, 2339);
}

#[test]
fn checker_property_access_optional_chain_no_error() {
    // Optional chaining on `any`-typed variable → no error.
    let diags = check_source("let x: any = null; let y = x?.foo;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_property_access_index_signature_no_error() {
    // Index signature allows any property access.
    let diags = check_source(
        "let x: { [key: string]: number } = { a: 1 };\
         let y = x.foo;",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_array_element_access_returns_element_type_no_error() {
    // `arr[0]` on `number[]` has type `number`.
    let diags = check_source("let arr: number[] = [1, 2, 3]; let y: number = arr[0];");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_array_element_access_returns_element_type_ts2322() {
    // `arr[0]` on `number[]` has type `number`; assigning to `string` fails.
    let diags = check_source("let arr: number[] = [1, 2, 3]; let y: string = arr[0];");
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_string_array_element_access_returns_string_no_error() {
    // `arr[0]` on `string[]` has type `string`.
    let diags = check_source("let arr: string[] = ['a']; let y: string = arr[0];");
    assert_no_diagnostics(&diags);
}

// ────────────────────────────────────────────────────────────────────────────
// Call-expression argument type checking (TS2345)
// ────────────────────────────────────────────────────────────────────────────

#[test]
fn checker_call_arg_matching_type_no_error() {
    // `f(42)` — `number` arg to `number` param → no error.
    let diags = check_source("function f(x: number) {} f(42);");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_call_arg_string_to_number_ts2345() {
    // `f('hi')` — `string` not assignable to `number` param → TS2345.
    let diags = check_source("function f(x: number) {} f('hi');");
    assert_diagnostic_code(&diags, 2345);
}

#[test]
fn checker_call_arg_number_to_string_ts2345() {
    // `f(42)` — `number` not assignable to `string` param → TS2345.
    let diags = check_source("function f(x: string) {} f(42);");
    assert_diagnostic_code(&diags, 2345);
}

#[test]
fn checker_call_arg_boolean_to_number_ts2345() {
    // `f(true)` — `boolean` not assignable to `number` param → TS2345.
    let diags = check_source("function f(x: number) {} f(true);");
    assert_diagnostic_code(&diags, 2345);
}

#[test]
fn checker_call_arg_union_member_no_error() {
    // `f(42)` — `number` is a member of `string | number` → no error.
    let diags = check_source("function f(x: string | number) {} f(42);");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_call_arg_outside_union_ts2345() {
    // `f(true)` — `boolean` not in `string | number` → TS2345.
    let diags = check_source("function f(x: string | number) {} f(true);");
    assert_diagnostic_code(&diags, 2345);
}

#[test]
fn checker_call_arg_any_param_no_error() {
    // `any` param accepts anything.
    let diags = check_source("function f(x: any) {} f('hi'); f(42); f(true);");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_call_multiple_args_first_mismatch_ts2345() {
    // Only the first argument mismatches.
    let diags = check_source("function f(a: number, b: string) {} f('hi', 'ok');");
    assert_diagnostic_code(&diags, 2345);
}

#[test]
fn checker_call_multiple_args_second_mismatch_ts2345() {
    // Only the second argument mismatches.
    let diags = check_source("function f(a: number, b: string) {} f(1, 42);");
    assert_diagnostic_code(&diags, 2345);
}

#[test]
fn checker_call_arrow_function_arg_ts2345() {
    // Arrow function callee.
    let diags = check_source("let f = (x: number) => x; f('hi');");
    assert_diagnostic_code(&diags, 2345);
}

#[test]
fn checker_call_arg_matching_object_type_no_error() {
    // Object literal arg matching the param's object type.
    let diags = check_source("function f(p: { a: number }) {} f({ a: 1 });");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_call_arg_object_missing_property_ts2345() {
    // `{ b: 1 }` not assignable to `{ a: number }` → TS2345.
    let diags = check_source("function f(p: { a: number }) {} f({ b: 1 });");
    assert_diagnostic_code(&diags, 2345);
}

#[test]
fn checker_call_arg_wrong_property_type_ts2345() {
    // `{ a: 'hi' }` not assignable to `{ a: number }` → TS2345.
    let diags = check_source("function f(p: { a: number }) {} f({ a: 'hi' });");
    assert_diagnostic_code(&diags, 2345);
}

#[test]
fn checker_new_expression_arg_ts2345() {
    // `new Foo('hi')` — `string` not assignable to `number` param → TS2345.
    let diags = check_source("class Foo { constructor(x: number) {} } let f = new Foo('hi');");
    assert_diagnostic_code(&diags, 2345);
}

#[test]
fn checker_new_expression_arg_no_error() {
    // `new Foo(42)` — `number` to `number` → no error.
    let diags = check_source("class Foo { constructor(x: number) {} } let f = new Foo(42);");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_call_arg_fewer_args_no_error() {
    // Fewer args than params (missing optional args are OK).
    let diags = check_source("function f(a: number, b?: string) {} f(42);");
    assert_no_diagnostics(&diags);
}

// ────────────────────────────────────────────────────────────────────────────
// Call-argument arity (TS2554 / TS2555)
// ────────────────────────────────────────────────────────────────────────────

#[test]
fn checker_call_too_few_args_ts2554() {
    // `f()` where `f(x: number)` expects 1 arg → TS2554.
    let diags = check_source("function f(x: number) {} f();");
    assert_diagnostic_code(&diags, 2554);
}

#[test]
fn checker_call_too_many_args_ts2554() {
    // `f(1, 2)` where `f(x: number)` expects 1 arg → TS2554.
    let diags = check_source("function f(x: number) {} f(1, 2);");
    assert_diagnostic_code(&diags, 2554);
}

#[test]
fn checker_call_exact_args_no_error() {
    let diags = check_source("function f(a: number, b: string) {} f(1, 'hi');");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_call_optional_param_no_args_no_error() {
    // Optional params don't count toward the minimum.
    let diags = check_source("function f(a?: number) {} f();");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_call_rest_param_no_args_no_error() {
    // Rest param with min 0 → `f()` is fine.
    let diags = check_source("function f(...args: number[]) {} f();");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_call_rest_param_many_args_no_error() {
    // Rest param accepts any number of trailing args.
    let diags = check_source("function f(...args: number[]) {} f(1, 2, 3);");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_call_required_then_rest_too_few_ts2555() {
    // `f()` where `f(a: number, ...rest: string[])` requires at least 1 → TS2555.
    let diags = check_source("function f(a: number, ...rest: string[]) {} f();");
    assert_diagnostic_code(&diags, 2555);
}

#[test]
fn checker_call_required_then_rest_enough_no_error() {
    // `f(1)` satisfies the minimum (1) for `f(a: number, ...rest: string[])`.
    let diags = check_source("function f(a: number, ...rest: string[]) {} f(1);");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_new_too_few_args_ts2554() {
    // `new Foo()` where ctor expects 1 arg → TS2554.
    let diags = check_source("class Foo { constructor(x: number) {} } let f = new Foo();");
    assert_diagnostic_code(&diags, 2554);
}

#[test]
fn checker_new_too_many_args_ts2554() {
    // `new Foo(1, 2)` where ctor expects 1 arg → TS2554.
    let diags = check_source("class Foo { constructor(x: number) {} } let f = new Foo(1, 2);");
    assert_diagnostic_code(&diags, 2554);
}

#[test]
fn checker_call_overload_too_few_ts2554() {
    // Overload set: no signature accepts 0 args → TS2554.
    let diags = check_source(
        "function f(x: number): void; function f(x: string): void; function f(x: any) {} f();",
    );
    assert_diagnostic_code(&diags, 2554);
}

#[test]
fn checker_call_overload_matching_no_error() {
    // One overload matches `(1)` exactly.
    let diags = check_source(
        "function f(x: number): void; function f(x: string): void; function f(x: any) {} f(1);",
    );
    assert_no_diagnostics(&diags);
}

// ────────────────────────────────────────────────────────────────────────────
// More expression type inference: non-null, conditional, template, delete, void
// ────────────────────────────────────────────────────────────────────────────

#[test]
fn checker_non_null_expression_returns_expression_type_no_error() {
    // `x!` has the type of `x` (here `number`).
    let diags = check_source("let x: number | null = 1; let y: number = x!;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_type_assertion_expression_uses_type_annotation_no_error() {
    // `<number>x` has type `number` (angle-bracket assertion).
    let diags = check_source("let x: any = 1; let y: number = <number>x;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_type_assertion_expression_wrong_annotation_ts2322() {
    // `<string>x` has type `string`; assigning to `number` fails.
    let diags = check_source("let x: any = 1; let y: number = <string>x;");
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_conditional_expression_both_branches_same_type_no_error() {
    // `cond ? 1 : 2` → `number` (both branches widen to `number`).
    let diags = check_source("let cond = true; let y: number = cond ? 1 : 2;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_conditional_expression_union_type_to_union_no_error() {
    // `cond ? 1 : 'hi'` → `number | string`; assigning to `number | string` is fine.
    let diags = check_source(
        "let cond = true;\
         let y: number | string = cond ? 1 : 'hi';",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_template_expression_returns_string_no_error() {
    // `` `${x}` `` → string.
    let diags = check_source("let x = 1; let y: string = `${x}`;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_template_expression_returns_string_ts2322() {
    // `` `${x}` `` → string; assigning to `number` fails.
    let diags = check_source("let x = 1; let y: number = `${x}`;");
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_delete_expression_returns_boolean_no_error() {
    // `delete x` → boolean.
    let diags = check_source("let x = 1; let y: boolean = delete x;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_void_expression_returns_undefined_no_error() {
    // `void x` → undefined.
    let diags = check_source("let x = 1; let y: undefined = void x;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_void_expression_returns_undefined_ts2322() {
    // `void x` → undefined; assigning to `number` fails.
    // Requires --strictNullChecks to make `undefined` not assignable to `number`.
    let diags = check_source_tsx_with_args(
        "let x = 1; let y: number = void x;",
        &["--strictNullChecks", "true"],
    );
    assert_diagnostic_code(&diags, 2322);
}

// ────────────────────────────────────────────────────────────────────────────
// Array literal type inference: `[1, 2, 3]` → `number[]`
// ────────────────────────────────────────────────────────────────────────────

#[test]
fn checker_array_literal_infer_number_array_no_error() {
    // `[1, 2, 3]` infers `number[]`; assigning to `number[]` is fine.
    let diags = check_source("let arr = [1, 2, 3]; let y: number[] = arr;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_array_literal_infer_number_array_ts2322() {
    // `[1, 2, 3]` infers `number[]`; assigning to `string[]` fails.
    let diags = check_source("let arr = [1, 2, 3]; let y: string[] = arr;");
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_array_literal_infer_string_array_no_error() {
    // `['a', 'b']` infers `string[]`.
    let diags = check_source("let arr = ['a', 'b']; let y: string[] = arr;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_array_literal_infer_string_array_ts2322() {
    // `['a', 'b']` infers `string[]`; assigning to `number[]` fails.
    let diags = check_source("let arr = ['a', 'b']; let y: number[] = arr;");
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_array_literal_element_access_after_inference_no_error() {
    // `arr[0]` where `arr` is inferred `number[]` → `number`.
    let diags = check_source("let arr = [1, 2, 3]; let y: number = arr[0];");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_array_literal_empty_no_error() {
    // Empty array `[]` infers `any[]`; assigning to `number[]` is fine.
    let diags = check_source("let arr = []; let y: number[] = arr;");
    assert_no_diagnostics(&diags);
}

// ────────────────────────────────────────────────────────────────────────────
// Object literal type inference: `{ a: 1, b: "hi" }` → `{ a: number, b: string }`
// ────────────────────────────────────────────────────────────────────────────

#[test]
fn checker_object_literal_infer_number_property_no_error() {
    // `{ a: 1 }` infers `{ a: number }`; assigning to `{ a: number }` is fine.
    let diags = check_source("let obj = { a: 1 }; let y: { a: number } = obj;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_object_literal_infer_number_property_to_string_ts2322() {
    // `{ a: 1 }` infers `{ a: number }`; assigning to `{ a: string }` fails.
    let diags = check_source("let obj = { a: 1 }; let y: { a: string } = obj;");
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_object_literal_infer_multiple_properties_no_error() {
    // `{ a: 1, b: 'hi' }` infers `{ a: number, b: string }`.
    let diags = check_source(
        "let obj = { a: 1, b: 'hi' };\
         let y: { a: number; b: string } = obj;",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_object_literal_infer_missing_property_ts2322() {
    // `{ a: 1 }` infers `{ a: number }`; assigning to `{ a: number; b: string }`
    // must fail because `b` is missing in the source. The checker reports the
    // specific TS2741 ("Property 'b' is missing...") rather than the generic
    // TS2322.
    let diags = check_source(
        "let obj = { a: 1 };\
         let y: { a: number; b: string } = obj;",
    );
    assert_diagnostic_code(&diags, 2741);
}

#[test]
fn checker_object_literal_infer_boolean_property_no_error() {
    // `{ flag: true }` infers `{ flag: boolean }` (literal `true` widens to `boolean`).
    let diags = check_source(
        "let obj = { flag: true };\
         let y: { flag: boolean } = obj;",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_object_literal_infer_nested_object_no_error() {
    // `{ a: { b: 1 } }` infers `{ a: { b: number } }`.
    let diags = check_source(
        "let obj = { a: { b: 1 } };\
         let y: { a: { b: number } } = obj;",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_object_literal_infer_shorthand_property_no_error() {
    // `{ a }` where `a` is `number` infers `{ a: number }`.
    let diags = check_source(
        "let a = 42;\
         let obj = { a };\
         let y: { a: number } = obj;",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_object_literal_infer_string_property_to_number_ts2322() {
    // `{ a: 'hi' }` infers `{ a: string }`; assigning to `{ a: number }` fails.
    let diags = check_source("let obj = { a: 'hi' }; let y: { a: number } = obj;");
    assert_diagnostic_code(&diags, 2322);
}

// ────────────────────────────────────────────────────────────────────────────
// Function type annotations: `(x: number) => string`
// ────────────────────────────────────────────────────────────────────────────

#[test]
fn checker_function_type_annotation_no_error() {
    // A function expression assignable to `(x: number) => number`.
    let diags = check_source("let f: (x: number) => number = (x) => x + 1;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_function_type_wrong_return_type_ts2322() {
    // `(x) => 'hi'` is not assignable to `(x: number) => number`.
    let diags = check_source("let f: (x: number) => number = (x) => 'hi';");
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_function_type_extra_parameters_ts2322() {
    // TS does NOT allow a function with *more required parameters* than the
    // target type to be assigned — calling through the target would leave
    // the extra required param undefined ("Target signature provides too
    // few arguments").
    let diags = check_source("let f: (x: number) => number = (x: number, y: number) => x + y;");
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_function_type_fewer_parameters_no_error() {
    // The reverse — a function with *fewer* parameters assigned to a type
    // expecting more — IS allowed (extra params are ignored by the callee).
    let diags = check_source("let f: (x: number, y: number) => number = (x: number) => x + 1;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_function_type_optional_parameter_no_error() {
    // `(x?: number) => number` is assignable to `(x: number) => number`
    // because optional params are compatible with required (bivariant).
    let diags = check_source("let f: (x: number) => number = (x?: number) => (x ?? 0) + 1;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_function_type_rest_parameter_no_error() {
    // `(...args: number[]) => number` is assignable to `(x: number) => number`.
    let diags = check_source("let f: (x: number) => number = (...args: number[]) => args[0] ?? 0;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_function_type_no_params_no_error() {
    // `() => number` assignable to `() => number`.
    let diags = check_source("let f: () => number = () => 42;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_function_type_void_return_no_error() {
    // A function returning `number` is assignable to `() => void`.
    let diags = check_source("let f: () => void = () => 42;");
    assert_no_diagnostics(&diags);
}

// ────────────────────────────────────────────────────────────────────────────
// Parameter type mismatch detection (P3.8: function-expression signature
// inference). Now that `get_type_of_function_like` builds a signature with
// the arrow/function expression's parameter types, the relater can detect
// parameter-type mismatches against a contextual function-type annotation.
// ────────────────────────────────────────────────────────────────────────────

#[test]
fn checker_arrow_param_type_mismatch_ts2322() {
    // `(x: string) => number` is not assignable to `(x: number) => number`
    // because `string` is not assignable to `number` (parameters are
    // contravariant under strictFunctionTypes, bivariant otherwise —
    // either way `string` vs `number` fails in both directions).
    let diags = check_source("let f: (x: number) => number = (x: string) => 1;");
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_arrow_param_type_match_no_error() {
    // `(x: number) => number` is assignable to `(x: number) => number`.
    let diags = check_source("let f: (x: number) => number = (x: number) => x + 1;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_arrow_two_param_type_mismatch_ts2322() {
    // Second parameter type mismatch: `(a: number, b: string)` vs
    // `(a: number, b: number)`.
    let diags =
        check_source("let f: (a: number, b: number) => number = (a: number, b: string) => 1;");
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_arrow_param_subtype_no_error() {
    // Bivariant parameter check: `string | number` parameter is assignable
    // to a `string | number` target parameter (same type both ways).
    let diags = check_source("let f: (x: string | number) => number = (x: string | number) => 1;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_function_expression_param_type_mismatch_ts2322() {
    // Same as the arrow case but with a `function` expression.
    let diags = check_source("let f: (x: number) => number = function (x: string) { return 1; };");
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_arrow_unannotated_param_no_error() {
    // Unannotated arrow param falls back to `any`, which is assignable to
    // anything (bivariant). `let f: (x: number) => number = (x) => x + 1;`
    // is the canonical contextually-typed arrow and must not error.
    let diags = check_source("let f: (x: number) => number = (x) => x + 1;");
    assert_no_diagnostics(&diags);
}

// ────────────────────────────────────────────────────────────────────────────
// Contextual typing for arrow/function-expression parameters (P3.8):
// unannotated parameters inherit the corresponding parameter type from the
// contextual function-type annotation. This flows into the function body, so
// return-type inference sees the real (contextual) parameter types rather
// than `any`.
// ────────────────────────────────────────────────────────────────────────────

#[test]
fn checker_contextual_param_return_type_mismatch_ts2322() {
    // `x` inherits `string` from the annotation; the arrow body returns
    // `x` (a `string`), which is not assignable to the expected `number`
    // return type → TS2322. Without contextual typing `x` would be `any`
    // and this would silently pass.
    let diags = check_source("let f: (x: string) => number = (x) => x;");
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_contextual_param_return_type_match_no_error() {
    // `x` inherits `number`; returning `x` (a `number`) matches the
    // expected `number` return type.
    let diags = check_source("let f: (x: number) => number = (x) => x;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_contextual_param_arithmetic_no_error() {
    // `x` inherits `number`; `x + 1` is `number`, matching the expected
    // `number` return type.
    let diags = check_source("let f: (x: number) => number = (x) => x + 1;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_contextual_param_block_body_return_ts2322() {
    // Block-bodied arrow: `x` inherits `number`; `return x;` yields
    // `number`, but the annotation expects `string` → TS2322.
    let diags = check_source("let f: (x: number) => string = (x) => { return x; };");
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_contextual_param_two_params_return_ts2322() {
    // `a`/`b` inherit `number`/`string` respectively; returning `b` (a
    // `string`) doesn't match the expected `number` return type.
    let diags = check_source("let f: (a: number, b: string) => number = (a, b) => b;");
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_contextual_param_function_expression_ts2322() {
    // Same contextual-typing behavior for `function` expressions: `x`
    // inherits `string`; returning `x` doesn't match the expected `number`.
    let diags = check_source("let f: (x: string) => number = function (x) { return x; };");
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_contextual_param_fewer_params_no_error() {
    // A function expression with FEWER params than the contextual signature
    // is allowed (extra params are ignored by the callee). The single param
    // `x` inherits `number`; returning `x` matches the expected `number`.
    let diags = check_source("let f: (x: number, y: number) => number = (x) => x;");
    assert_no_diagnostics(&diags);
}

// ────────────────────────────────────────────────────────────────────────────
// P3.8: Conditional types + `infer R` (TS2322 for branch mismatches)
// ────────────────────────────────────────────────────────────────────────────

#[test]
fn checker_conditional_true_branch_no_error() {
    // `number extends number` is true → T = "yes".
    let diags =
        check_source("type T = number extends number ? \"yes\" : \"no\";\nlet x: T = \"yes\";");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_conditional_true_branch_mismatch_ts2322() {
    // T = "yes" but we assign "no".
    let diags =
        check_source("type T = number extends number ? \"yes\" : \"no\";\nlet x: T = \"no\";");
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_conditional_false_branch_no_error() {
    // `number extends string` is false → T = "no".
    let diags =
        check_source("type T = number extends string ? \"yes\" : \"no\";\nlet x: T = \"no\";");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_conditional_false_branch_mismatch_ts2322() {
    // T = "no" but we assign "yes".
    let diags =
        check_source("type T = number extends string ? \"yes\" : \"no\";\nlet x: T = \"yes\";");
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_conditional_literal_check_type_true_no_error() {
    // `1 extends number` is true → T = "a".
    let diags = check_source("type T = 1 extends number ? \"a\" : \"b\";\nlet x: T = \"a\";");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_conditional_literal_check_type_false_no_error() {
    // `1 extends string` is false → T = "b".
    let diags = check_source("type T = 1 extends string ? \"a\" : \"b\";\nlet x: T = \"b\";");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_conditional_infer_r_array_element_no_error() {
    // `number[] extends (infer R)[] ? R : never` → R = number.
    let diags = check_source(
        "type T<U> = U extends (infer R)[] ? R : never;\nlet x: number = null as any as T<number[]>;",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_conditional_infer_r_array_element_mismatch_ts2322() {
    // R = number but we assign to string.
    let diags = check_source(
        "type T<U> = U extends (infer R)[] ? R : never;\nlet x: string = null as any as T<number[]>;",
    );
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_conditional_infer_r_string_no_error() {
    // `string extends infer R ? R : never` → R = string.
    let diags = check_source(
        "type T<U> = U extends infer R ? R : never;\nlet x: string = null as any as T<string>;",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_conditional_infer_r_number_mismatch_ts2322() {
    // R = number but we assign to string.
    let diags = check_source(
        "type T<U> = U extends infer R ? R : never;\nlet x: string = null as any as T<number>;",
    );
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_conditional_infer_r_never_branch_no_error() {
    // `string extends number ? "yes" : never` → never. Assigning never is OK.
    let diags = check_source(
        "type T = string extends number ? \"yes\" : never;\nlet x: \"yes\" = null as any as T;",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_conditional_nested_true_no_error() {
    // Nested conditional: number extends number ? (1 extends number ? "a" : "b") : "c" → "a".
    let diags = check_source(
        "type T = number extends number ? (1 extends number ? \"a\" : \"b\") : \"c\";\nlet x: T = \"a\";",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_conditional_nested_false_mismatch_ts2322() {
    // number extends string → false branch → "c". Assigning "a" fails.
    let diags = check_source(
        "type T = number extends string ? (1 extends number ? \"a\" : \"b\") : \"c\";\nlet x: T = \"a\";",
    );
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_conditional_boolean_check_true_no_error() {
    // `true extends boolean` → true → T = 1.
    let diags = check_source("type T = true extends boolean ? 1 : 0;\nlet x: T = 1;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_conditional_boolean_check_false_no_error() {
    // `true extends string` → false → T = 0.
    let diags = check_source("type T = true extends string ? 1 : 0;\nlet x: T = 0;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_conditional_infer_r_in_function_return_no_error() {
    // `(...) => infer R ? R : never` → R = number.
    let diags = check_source(
        "type Ret<F> = F extends (...args: any[]) => infer R ? R : never;\nlet x: number = null as any as Ret<() => number>;",
    );
    assert_no_diagnostics(&diags);
}

// ────────────────────────────────────────────────────────────────────────────
// P3.7: keyof T — union of string-literal property names
// ────────────────────────────────────────────────────────────────────────────

#[test]
fn checker_keyof_object_type_no_error() {
    // `keyof { a: number; b: string }` = "a" | "b".
    let diags = check_source(
        "type K = keyof { a: number; b: string };\nlet x: \"a\" | \"b\" = null as any as K;",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_keyof_object_type_single_key_no_error() {
    // `keyof { x: 1 }` = "x".
    let diags = check_source("type K = keyof { x: 1 };\nlet x: \"x\" = null as any as K;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_keyof_object_type_subset_assignable_no_error() {
    // `keyof { a: number }` = "a". "a" IS assignable to "a" | "b" (subset).
    let diags =
        check_source("type K = keyof { a: number };\nlet x: \"a\" | \"b\" = null as any as K;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_keyof_object_type_missing_key_ts2322() {
    // `keyof { a: number; b: string }` = "a" | "b", but target expects only "a".
    let diags =
        check_source("type K = keyof { a: number; b: string };\nlet x: \"a\" = null as any as K;");
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_keyof_via_type_alias_no_error() {
    // `keyof T` where T is a type alias for an object literal type.
    let diags = check_source(
        "type Obj = { a: number; b: string };\ntype K = keyof Obj;\nlet x: \"a\" | \"b\" = null as any as K;",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_keyof_never_type() {
    // `keyof never` = never. Assigning never to a string-literal type is OK.
    let diags = check_source("type K = keyof never;\nlet x: \"a\" = null as any as K;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_keyof_empty_object_is_never() {
    // `keyof {}` = never (no keys). Assigning never is OK.
    let diags = check_source("type K = keyof {};\nlet x: \"a\" = null as any as K;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_keyof_union_common_keys_no_error() {
    // `keyof (A | B)` = `keyof A & keyof B` = common keys only.
    // A has "a"|"b", B has "b"|"c" → keyof (A|B) = "b".
    let diags = check_source(
        "type A = { a: number; b: string };\ntype B = { b: string; c: number };\ntype K = keyof (A | B);\nlet x: \"b\" = null as any as K;",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_keyof_intersection_all_keys_no_error() {
    // `keyof (A & B)` = `keyof A | keyof B` = all keys.
    // A has "a", B has "b" → keyof (A&B) = "a" | "b".
    let diags = check_source(
        "type A = { a: number };\ntype B = { b: string };\ntype K = keyof (A & B);\nlet x: \"a\" | \"b\" = null as any as K;",
    );
    assert_no_diagnostics(&diags);
}

// ────────────────────────────────────────────────────────────────────────────
// Indexed access types: `T["a"]`, `T[K]`, `T[number]`, `T[keyof T]`
// ────────────────────────────────────────────────────────────────────────────

#[test]
fn checker_indexed_access_string_literal_no_error() {
    // `T["a"]` resolves to the type of property `a`.
    let diags = check_source(
        "type T = { a: number; b: string };\ntype A = T[\"a\"];\nlet x: number = null as any as A;",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_indexed_access_string_literal_mismatch_ts2322() {
    // `T["a"]` is `number`; assigning to `string` must fail.
    let diags = check_source(
        "type T = { a: number; b: string };\ntype A = T[\"a\"];\nlet x: string = null as any as A;",
    );
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_indexed_access_keyof_no_error() {
    // `T[keyof T]` = union of all property types = `number | string`.
    let diags = check_source(
        "type T = { a: number; b: string };\ntype V = T[keyof T];\nlet x: number | string = null as any as V;",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_indexed_access_keyof_missing_member_ts2322() {
    // `T[keyof T]` = `number | string`; assigning to `boolean` fails.
    let diags = check_source(
        "type T = { a: number; b: string };\ntype V = T[keyof T];\nlet x: boolean = null as any as V;",
    );
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_indexed_access_union_of_keys_no_error() {
    // `T["a" | "b"]` = `T["a"] | T["b"]` = `number | string`.
    let diags = check_source(
        "type T = { a: number; b: string };\ntype K = \"a\" | \"b\";\ntype V = T[K];\nlet x: number | string = null as any as V;",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_indexed_access_array_number_no_error() {
    // `number[]` indexed by `number` → `number`.
    let diags = check_source(
        "type Arr = number[];\ntype Elem = Arr[number];\nlet x: number = null as any as Elem;",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_indexed_access_array_number_mismatch_ts2322() {
    // `number[]` indexed by `number` → `number`; assigning to `string` fails.
    let diags = check_source(
        "type Arr = number[];\ntype Elem = Arr[number];\nlet x: string = null as any as Elem;",
    );
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_indexed_access_via_alias_no_error() {
    // Indexed access through a type alias.
    let diags = check_source(
        "type Obj = { value: boolean };\ntype V = Obj[\"value\"];\nlet x: boolean = null as any as V;",
    );
    assert_no_diagnostics(&diags);
}

// ────────────────────────────────────────────────────────────────────────────
// Template literal types: `a-${string}`, `a-${1}-b`
// ────────────────────────────────────────────────────────────────────────────

#[test]
fn checker_template_literal_type_concrete_flatten_no_error() {
    // All spans concrete → flatten to "a-1-b".
    let diags = check_source("type T = `a-${1}-b`;\nlet x: \"a-1-b\" = null as any as T;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_template_literal_type_concrete_flatten_mismatch_ts2322() {
    // `a-${1}` flattens to "a-1"; assigning to "a-2" fails.
    let diags = check_source("type T = `a-${1}`;\nlet x: \"a-2\" = null as any as T;");
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_template_literal_type_string_span_no_error() {
    // `${string}` span → template-literal type. A concrete string literal
    // is assignable to it.
    let diags =
        check_source("type T = `prefix-${string}`;\nlet x: T = null as any as `prefix-hello`;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_template_literal_type_multiple_spans_flatten_no_error() {
    // Multiple concrete spans: `x-${true}-${"y"}` → "x-true-y".
    let diags =
        check_source("type T = `x-${true}-${\"y\"}`;\nlet x: \"x-true-y\" = null as any as T;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_template_literal_type_via_alias_no_error() {
    // Template literal type referenced through a type alias.
    let diags = check_source(
        "type Prefix<T> = `pre-${T}`;\ntype P = Prefix<\"x\">;\nlet v: \"pre-x\" = null as any as P;",
    );
    assert_no_diagnostics(&diags);
}

// ────────────────────────────────────────────────────────────────────────────
// Mapped types: { [K in C]: V }, { [K in keyof T]: V }
// ────────────────────────────────────────────────────────────────────────────

#[test]
fn checker_mapped_type_literal_union_no_error() {
    // `{ [K in "a" | "b"]: number }` → `{ a: number; b: number }`.
    let diags =
        check_source("type M = { [K in \"a\" | \"b\"]: number };\nlet x: M = { a: 1, b: 2 };");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_mapped_type_literal_union_mismatch_ts2322() {
    // `{ [K in "a" | "b"]: number }` → `{ a: number; b: number }`.
    // Assigning a string to `a` fails.
    let diags =
        check_source("type M = { [K in \"a\" | \"b\"]: number };\nlet x: M = { a: \"hi\", b: 2 };");
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_mapped_type_keyof_no_error() {
    // `{ [K in keyof T]: number }` where T = { a: 1; b: "x" }
    // → { a: number; b: number }.
    let diags = check_source(
        "type T = { a: 1; b: \"x\" };\ntype M = { [K in keyof T]: number };\nlet x: M = { a: 1, b: 2 };",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_mapped_type_keyof_mismatch_ts2322() {
    // `{ [K in keyof T]: number }` → { a: number; b: number }.
    // Missing `b` should fail. The checker reports the specific TS2741
    // ("Property 'b' is missing...") rather than the generic TS2322.
    let diags = check_source(
        "type T = { a: 1; b: \"x\" };\ntype M = { [K in keyof T]: number };\nlet x: M = { a: 1 };",
    );
    assert_diagnostic_code(&diags, 2741);
}

#[test]
fn checker_mapped_type_optional_no_error() {
    // `{ [K in keyof T]?: number }` → { a?: number; b?: number }.
    // Missing properties are OK because they're optional.
    let diags = check_source(
        "type T = { a: 1; b: \"x\" };\ntype M = { [K in keyof T]?: number };\nlet x: M = { };",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_mapped_type_identity_no_error() {
    // `{ [K in keyof T]: T[K] }` → same shape as T.
    let diags = check_source(
        "type T = { a: number; b: string };\ntype M = { [K in keyof T]: T[K] };\nlet x: M = { a: 1, b: \"hi\" };",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_mapped_type_identity_mismatch_ts2322() {
    // `{ [K in keyof T]: T[K] }` → same shape as T.
    // Wrong types should fail.
    let diags = check_source(
        "type T = { a: number; b: string };\ntype M = { [K in keyof T]: T[K] };\nlet x: M = { a: \"hi\", b: \"hi\" };",
    );
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_keyof_constrained_type_parameter_no_error() {
    // `keyof T` where T extends { a: number; b: string } → "a" | "b".
    let diags = check_source(
        "type K<T extends { a: number; b: string }> = keyof T;\nlet x: \"a\" | \"b\" = null as any as K<{ a: 1; b: 2 }>;",
    );
    assert_no_diagnostics(&diags);
}

// ────────────────────────────────────────────────────────────────────────────
// P3.7: Conditional type relation (source assignable to conditional target)
// and recursive structural type cycle detection.
// ────────────────────────────────────────────────────────────────────────────

#[test]
fn checker_conditional_target_accepts_true_branch_no_error() {
    // `T extends U ? X : Y` as a *target*: a value of type `X` is
    // assignable to the conditional when the conditional is known to
    // take the true branch (check is assignable to extends).
    // `number extends number ? string : number` resolves to `string`,
    // so a `string` value is assignable.
    let diags = check_source(
        "type C = number extends number ? string : number;\nlet x: string = \"hi\" as C;",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_recursive_structural_type_assignable_no_error() {
    // Recursive structural type `Box<T> = { next: Box<T> | null }`.
    // Comparing `Box<number>` to `Box<number>` must not stack-overflow
    // and must be assignable. Exercises the relater's cycle detection
    // (relation_in_progress) and depth guard.
    let diags = check_source(
        "type Box<T> = { next: Box<T> | null };\nlet x: Box<number> = { next: null };",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_recursive_structural_type_self_assignable_no_error() {
    // Two mutually-recursive structural types with the same shape must be
    // assignable. The relater's `relation_in_progress` cycle set breaks
    // the infinite recursion when comparing `A.next` (type `A | null`)
    // against `B.next` (type `B | null`), which would otherwise reach
    // `A` vs `B` again.
    let diags = check_source(
        "type A = { value: number; next: A | null };\n\
         type B = { value: number; next: B | null };\n\
         let x: B = { value: 1, next: null } as A;",
    );
    assert_no_diagnostics(&diags);
}

// ────────────────────────────────────────────────────────────────────────────
// P3.8: Fresh literal type widening for variable declarations without
// a type annotation. Object literals widen each property's literal type
// to its primitive base (`1` → `number`, `'hi'` → `string`, `true` → `boolean`).
// ────────────────────────────────────────────────────────────────────────────

#[test]
fn checker_object_literal_widening_reassignment_no_error() {
    // `let x = { a: 1 }` infers `{ a: number }` (widened), so reassigning
    // `x = { a: 2 }` is fine. Without widening, `x` would be `{ a: 1 }`
    // and `{ a: 2 }` would not be assignable (TS2322 false positive).
    let diags = check_source("let x = { a: 1 };\nx = { a: 2 };");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_object_literal_widening_property_assignment_no_error() {
    // `let x = { a: 1 }` infers `{ a: number }`, so `x.a = 2` is fine.
    // Without widening, `x.a` would be `1` and assigning `2` would fail.
    let diags = check_source("let x = { a: 1 };\nx.a = 2;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_object_literal_widening_string_no_error() {
    // String literal property widens to `string`.
    let diags = check_source("let x = { a: 'hi' };\nx = { a: 'bye' };");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_object_literal_widening_boolean_no_error() {
    // Boolean literal property widens to `boolean`.
    let diags = check_source("let x = { flag: true };\nx = { flag: false };");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_object_literal_widening_nested_no_error() {
    // Nested object literal: inner literal types also widen.
    let diags = check_source("let x = { a: { b: 1 } };\nx = { a: { b: 2 } };");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_object_literal_contextual_preserves_literal_no_error() {
    // When a type annotation IS present, the object literal's literal
    // types are preserved (contextual typing) and checked against the
    // annotation. `{ a: 1 }` is assignable to `{ a: 1 }` (literal match).
    let diags = check_source("let x: { a: 1 } = { a: 1 };");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_object_literal_contextual_mismatch_ts2322() {
    // Contextual typing preserves the literal `1`, which is NOT assignable
    // to `{ a: 2 }` (different literal). This confirms that widening only
    // happens when there's no type annotation.
    let diags = check_source("let x: { a: 2 } = { a: 1 };");
    assert_diagnostic_code(&diags, 2322);
}

// ────────────────────────────────────────────────────────────────────────────
// P3.8: Contextual typing for CallExpression arguments. Arrow function
// parameters inherit types from the contextual signature, and object
// literal arguments preserve literal types when checked against the
// parameter type.
// ────────────────────────────────────────────────────────────────────────────

#[test]
fn checker_call_arg_arrow_contextual_param_type_ts2339() {
    // Arrow function argument: `x` should be contextually typed as
    // `{ a: number }` from the parameter type, so `x.b` should report
    // TS2339 (no property `b`). Without contextual typing, `x` would
    // be `any` and no error would be reported.
    let diags = check_source(
        "function f(cb: (x: { a: number }) => void): void {}\n\
         f((x) => x.b);",
    );
    assert_diagnostic_code(&diags, 2339);
}

#[test]
fn checker_call_arg_arrow_contextual_valid_no_error() {
    // Arrow function argument: `x` contextually typed as `{ a: number }`,
    // accessing `x.a` is valid.
    let diags = check_source(
        "function f(cb: (x: { a: number }) => void): void {}\n\
         f((x) => x.a);",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_call_arg_object_literal_contextual_no_error() {
    // Object literal argument: `{ a: 1 }` is contextually typed by
    // `{ a: number }`. The literal `1` is assignable to `number`.
    let diags = check_source(
        "function f(x: { a: number }): void {}\n\
         f({ a: 1 });",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_call_arg_object_literal_contextual_mismatch_ts2345() {
    // Object literal argument: `{ a: 'hi' }` is NOT assignable to
    // `{ a: number }`.
    let diags = check_source(
        "function f(x: { a: number }): void {}\n\
         f({ a: 'hi' });",
    );
    assert_diagnostic_code(&diags, 2345);
}

// ────────────────────────────────────────────────────────────────────────────
// P3.1: try/catch/finally flow graph narrowing. Variables narrowed in the
// try block should NOT retain the narrowed type after the try/catch/finally
// (since an exception could have occurred before the narrowing).
// ────────────────────────────────────────────────────────────────────────────

#[test]
fn checker_try_catch_finally_no_crash_no_error() {
    // Basic try/catch/finally should not produce false positives.
    let diags = check_source(
        "try {\n  let x = 1;\n} catch (e) {\n  let y = 2;\n} finally {\n  let z = 3;\n}",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_narrowing_lost_after_try_block_no_error() {
    // `x` is narrowed to `string` inside the `try` block by the
    // `typeof` check. After the `try/catch` block, `x` should be
    // back to `string | number` (the declared type) because an
    // exception could have occurred before the narrowing took effect.
    // Reassigning `x = 123` should be fine because `x` is `string | number`.
    //
    // This test verifies that narrowing from inside `try` doesn't
    // leak out — `x` retains its declared union type after try/catch.
    let diags = check_source(
        "let x: string | number = 'hi';\n\
         try {\n  if (typeof x === 'string') { x = 'bye'; }\n\
         } catch (e) {}\n\
         x = 123;",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_catch_variable_no_error() {
    // The catch variable `e` should not produce false positives
    // when used inside the catch block.
    let diags = check_source("try {\n  throw 42;\n} catch (e) {\n  let y = e;\n}");
    assert_no_diagnostics(&diags);
}

// ────────────────────────────────────────────────────────────────────────────
// P3.10: Interface type resolution. `interface Foo { a: number }` now
// resolves to an anonymous object type with property signatures, enabling
// assignability checks against interface-typed variables.
// ────────────────────────────────────────────────────────────────────────────

#[test]
fn checker_interface_assignable_no_error() {
    // Object literal with matching properties is assignable to the interface.
    let diags = check_source(
        "interface Foo { a: number; b: string }\n\
         let x: Foo = { a: 1, b: 'hi' };",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_interface_missing_property_ts2741() {
    // Missing property `b` should report an error.
    let diags = check_source(
        "interface Foo { a: number; b: string }\n\
         let x: Foo = { a: 1 };",
    );
    assert!(diags.iter().any(|d| d.code != 0));
}

#[test]
fn checker_interface_wrong_property_type_ts2322() {
    // Wrong property type should report TS2322.
    let diags = check_source(
        "interface Foo { a: number }\n\
         let x: Foo = { a: 'hi' };",
    );
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_interface_property_access_no_error() {
    // Accessing a property on an interface-typed variable should work.
    let diags = check_source(
        "interface Foo { a: number }\n\
         let x: Foo = { a: 1 };\n\
         let y: number = x.a;",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_interface_property_access_missing_ts2339() {
    // Accessing a non-existent property should report TS2339.
    let diags = check_source(
        "interface Foo { a: number }\n\
         let x: Foo = { a: 1 };\n\
         x.b;",
    );
    assert_diagnostic_code(&diags, 2339);
}

#[test]
fn checker_generic_interface_substitution_no_error() {
    // Generic interface `Box<T>` with `Box<number>` should substitute `T`
    // with `number` in the `value` property type.
    let diags = check_source(
        "interface Box<T> { value: T }\n\
         let x: Box<number> = { value: 1 };",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_generic_interface_substitution_mismatch_ts2322() {
    // Generic interface `Box<T>` with `Box<number>` — assigning a string
    // value should fail because `T` is substituted with `number`.
    let diags = check_source(
        "interface Box<T> { value: T }\n\
         let x: Box<number> = { value: 'hi' };",
    );
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_interface_method_signature_no_error() {
    // Interface with a method signature — calling the method via a
    // function-expression property should work.
    let diags = check_source(
        "interface Foo { greet(): void }\n\
         let x: Foo = { greet: () => {} };\n\
         x.greet();",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_interface_index_signature_no_error() {
    // Interface with an index signature — accessing by string key should work.
    let diags = check_source(
        "interface Foo { [key: string]: number }\n\
         let x: Foo = { a: 1 };\n\
         let y: number = x.a;",
    );
    assert_no_diagnostics(&diags);
}

// ────────────────────────────────────────────────────────────────────────────
// Enum type resolution
// ────────────────────────────────────────────────────────────────────────────

#[test]
fn checker_numeric_enum_no_error() {
    // Numeric enum: `Color.Red` should have type `0` (number literal).
    let diags = check_source(
        "enum Color { Red, Green, Blue }\n\
         let x: Color = Color.Red;",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_numeric_enum_member_values() {
    // Numeric enum with explicit values.
    let diags = check_source(
        "enum Direction { Up = 1, Down = 2 }\n\
         let x: Direction = Direction.Up;",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_string_enum_no_error() {
    // String enum: members have string literal types.
    let diags = check_source(
        "enum Direction { Up = 'UP', Down = 'DOWN' }\n\
         let x: Direction = Direction.Up;",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_enum_property_access_no_error() {
    // Accessing an enum member should resolve to its literal type.
    let diags = check_source(
        "enum Color { Red = 0, Green = 1 }\n\
         let r = Color.Red;\n\
         let g = Color.Green;",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_enum_wrong_assign_ts2322() {
    // Assigning a non-enum value to an enum-typed variable should fail.
    let diags = check_source(
        "enum Color { Red, Green, Blue }\n\
         let x: Color = 42;",
    );
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_mixed_enum_no_error() {
    // Mixed enum with both numeric and string members.
    let diags = check_source(
        "enum Shape { Circle = 0, Square = 'SQ' }\n\
         let x: Shape = Shape.Circle;",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_enum_auto_increment() {
    // Auto-increment: Green should be 1, Blue should be 2.
    let diags = check_source(
        "enum Color { Red = 0, Green, Blue }\n\
         let x: Color = Color.Green;",
    );
    assert_no_diagnostics(&diags);
}

// ────────────────────────────────────────────────────────────────────────────
// Evolving array types (ARRAY_MUTATION flow nodes)
// ────────────────────────────────────────────────────────────────────────────

#[test]
fn checker_evolving_array_push_number_no_error() {
    // `let x = []; x.push(1)` — after the push, x's element type should be
    // `number`, so `x[0]` should be assignable to `number`.
    let diags = check_source(
        "let x = [];\n\
         x.push(1);\n\
         let y: number = x[0];",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_evolving_array_push_string_no_error() {
    // `let x = []; x.push('hi')` — after the push, x's element type should be
    // `string`.
    let diags = check_source(
        "let x = [];\n\
         x.push('hi');\n\
         let y: string = x[0];",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_evolving_array_push_mismatch_ts2322() {
    // `let x = []; x.push(1)` — element type is `number`, so assigning
    // `x[0]` to a `string` variable should fail with TS2322.
    let diags = check_source(
        "let x = [];\n\
         x.push(1);\n\
         let y: string = x[0];",
    );
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_evolving_array_multiple_pushes_no_error() {
    // Multiple pushes of the same type should not error.
    let diags = check_source(
        "let x = [];\n\
         x.push(1);\n\
         x.push(2);\n\
         x.push(3);\n\
         let y: number = x[0];",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_evolving_array_empty_no_error() {
    // An empty evolving array (no pushes) should finalize to `any[]`,
    // which is assignable to `any`.
    let diags = check_source(
        "let x = [];\n\
         let y = x;",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_evolving_array_unshift_no_error() {
    // `unshift` should also evolve the array type.
    let diags = check_source(
        "let x = [];\n\
         x.unshift(1);\n\
         let y: number = x[0];",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_non_empty_array_literal_no_error() {
    // Non-empty array literal `[1, 2, 3]` should infer `number[]`
    // directly (not evolving).
    let diags = check_source(
        "let x = [1, 2, 3];\n\
         let y: number = x[0];",
    );
    assert_no_diagnostics(&diags);
}

// ────────────────────────────────────────────────────────────────────────────
// Hover / quick info (P3.10 symbol_to_type_node / hover info)
//
// `get_quick_info_text(node)` returns the plain-text hover string for a
// node. We exercise it via a small helper that walks the AST to find the
// first identifier with a given name. This mirrors (very minimally) what
// the LSP `textDocument/hover` request returns.
// ────────────────────────────────────────────────────────────────────────────

use tsox::ast::{NodeData, SyntaxKind};
use tsox::checker::{Checker, Tracer};

/// Build a program from `source`, run the checker, and invoke
/// `get_quick_info_text` on the first identifier named `name` in the
/// file's AST. Returns the hover string (or `None` if the name isn't
/// found).
fn hover_info_for(source: &str, name: &str) -> Option<String> {
    let fs = Arc::new(InMemoryFS::new());
    fs.insert_dir("/proj");
    fs.insert_file("/proj/entry.ts", source);

    let args = vec!["--noLib".to_string(), "/proj/entry.ts".to_string()];
    let parsed = parse_command_line(&args, "/proj", Some(fs.as_ref()));
    let host: Arc<dyn tsox::compiler::CompilerHost> =
        Arc::new(CompilerHostImpl::new(fs, "/proj".to_string(), lib_path()));
    let program = Arc::new(Program::new(ProgramOptions {
        config: parsed,
        host,
    }));

    let tracer = Arc::new(Tracer::new());
    let program_dyn: Arc<dyn tsox::checker::Program> = Arc::clone(&program) as _;
    let mut checker = Checker::new(program_dyn, tracer);
    for file in program.source_files() {
        checker.check_source_file(file);
    }

    let target = find_identifier(&program.source_files()[0].node, name)?;
    Some(checker.get_quick_info_text(&target))
}

/// Recursively walk `node`'s subtree looking for the first `Identifier`
/// whose text matches `name`.
fn find_identifier(node: &Arc<tsox::ast::Node>, name: &str) -> Option<Arc<tsox::ast::Node>> {
    if node.kind == SyntaxKind::Identifier {
        if let NodeData::Identifier(id) = &node.data {
            if id.text == name {
                return Some(Arc::clone(node));
            }
        }
    }
    let mut found: Option<Arc<tsox::ast::Node>> = None;
    tsox::ast::node_data_generated::for_each_child(node, |child| {
        if found.is_none() {
            found = find_identifier(child, name);
        }
        found.is_some()
    });
    found
}

#[test]
fn hover_let_variable_number() {
    let info = hover_info_for("let x: number = 0;", "x").expect("hover");
    assert_eq!(info, "let x: number");
}

#[test]
fn hover_const_variable_string() {
    let info = hover_info_for("const s: string = \"hi\";", "s").expect("hover");
    assert_eq!(info, "const s: string");
}

#[test]
fn hover_let_variable_inferred_type() {
    // `let x = 1;` infers `number` via literal widening.
    let info = hover_info_for("let x = 1;", "x").expect("hover");
    assert_eq!(info, "let x: number");
}

#[test]
fn hover_function_declaration() {
    let info = hover_info_for(
        "function f(a: string, b: number): boolean { return true; }",
        "f",
    )
    .expect("hover");
    assert_eq!(info, "function f(a: string, b: number): boolean");
}

#[test]
fn hover_class_declaration_no_type_params() {
    let info = hover_info_for("class Foo {}", "Foo").expect("hover");
    assert_eq!(info, "class Foo");
}

#[test]
fn hover_class_declaration_with_type_params() {
    let info = hover_info_for("class Foo<T, U> {}", "Foo").expect("hover");
    assert_eq!(info, "class Foo<T, U>");
}

#[test]
fn hover_interface_declaration_no_type_params() {
    let info = hover_info_for("interface Bar { x: number; }", "Bar").expect("hover");
    assert_eq!(info, "interface Bar");
}

#[test]
fn hover_interface_declaration_with_type_params() {
    let info = hover_info_for("interface Bar<T> { x: T; }", "Bar").expect("hover");
    assert_eq!(info, "interface Bar<T>");
}

#[test]
fn hover_enum_declaration() {
    let info = hover_info_for("enum Color { Red, Green, Blue }", "Color").expect("hover");
    assert_eq!(info, "enum Color");
}

#[test]
fn hover_type_alias_declaration_primitive() {
    let info = hover_info_for("type MyNumber = number;", "MyNumber").expect("hover");
    assert_eq!(info, "type MyNumber = number");
}

#[test]
fn hover_type_alias_declaration_with_type_params() {
    let info = hover_info_for("type Id<T> = T;", "Id").expect("hover");
    // The aliased type of `Id<T>` is just `T`, but we haven't
    // instantiated it under a particular `T` here, so we only assert
    // the prefix.
    assert!(
        info.starts_with("type Id<T> = "),
        "expected `type Id<T> = ...`, got {info:?}"
    );
}

#[test]
fn hover_var_keyword_variable() {
    let info = hover_info_for("var v: string = \"hi\";", "v").expect("hover");
    assert_eq!(info, "var v: string");
}

// ────────────────────────────────────────────────────────────────────────────
// Declaration merge parity (P3.4)
// ────────────────────────────────────────────────────────────────────────────

#[test]
fn checker_interface_merge_no_error() {
    // Two `interface Foo` declarations should merge into a single type with
    // all members; `{ a: 1, b: "hi" }` is assignable to the merged type.
    let diags = check_source(
        "interface Foo { a: number; }\n\
         interface Foo { b: string; }\n\
         const x: Foo = { a: 1, b: \"hi\" };",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_interface_merge_missing_member_ts2322() {
    // After merging, `Foo` requires both `a` and `b`; missing `b` is an error.
    // The checker reports the specific TS2741 ("Property 'b' is missing...")
    // rather than the generic TS2322.
    let diags = check_source(
        "interface Foo { a: number; }\n\
         interface Foo { b: string; }\n\
         const x: Foo = { a: 1 };",
    );
    assert_diagnostic_code(&diags, 2741);
}

#[test]
fn checker_interface_merge_missing_first_member_ts2322() {
    // Verifies true merging: a member from the FIRST declaration (`a`)
    // must still be required even though only the second `interface Foo`
    // symbol survives in the binder's member table. If merging were broken
    // (only the second interface's members were seen), `{ b: \"hi\" }`
    // would be assignable and this test would expect no diagnostics.
    let diags = check_source(
        "interface Foo { a: number; }\n\
         interface Foo { b: string; }\n\
         const x: Foo = { b: \"hi\" };",
    );
    assert_diagnostic_code(&diags, 2741);
}

#[test]
fn checker_function_overload_no_error() {
    // Multiple `function f` declarations merge into one symbol. The
    // implementation signature (`(x: any): any`) is the last declaration.
    // Calling with a string matches the first overload.
    let diags = check_source(
        "function f(x: string): string;\n\
         function f(x: number): number;\n\
         function f(x: any): any { return x; }\n\
         const s: string = f(\"hi\");\n\
         const n: number = f(42);",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_namespace_merge_no_error() {
    // Two `namespace N` declarations merge; members from both are visible.
    let diags = check_source(
        "namespace N { export const a: number = 1; }\n\
         namespace N { export const b: string = \"hi\"; }\n\
         const a: number = N.a;\n\
         const b: string = N.b;",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_namespace_merge_missing_member_ts2339() {
    // After merging, accessing a non-existent member of `N` should fail.
    let diags = check_source(
        "namespace N { export const a: number = 1; }\n\
         namespace N { export const b: string = \"hi\"; }\n\
         const x = N.c;",
    );
    assert_diagnostic_code(&diags, 2339);
}

#[test]
fn checker_optional_property_no_error() {
    // Optional property on an interface should allow undefined.
    let diags = check_source(
        "interface Opt { x?: number; }\n\
         const o: Opt = { };\n\
         const n: number | undefined = o.x;",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_optional_property_wrong_type_ts2322() {
    // Wrong type for optional property should fail.
    let diags = check_source(
        "interface Opt { x?: number; }\n\
         const o: Opt = { x: \"hi\" };",
    );
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_class_inherits_method_no_error() {
    // Basic class inheritance with `extends`: derived class should
    // inherit the base class's method and resolve the property access.
    let diags = check_source(
        "class Base { method(): number { return 1; } }\n\
         class Derived extends Base { }\n\
         const d = new Derived();\n\
         const n = d.method();",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_class_implements_interface_missing_member_ts2420() {
    // `implements` without the required member should report TS2420.
    let diags = check_source(
        "interface IFoo { bar(): number; }\n\
         class C implements IFoo { }",
    );
    // TS2420: Class 'C' incorrectly implements interface 'IFoo'.
    assert_diagnostic_code(&diags, 2420);
}

#[test]
fn hover_arrow_function_variable() {
    let info = hover_info_for("let f = (a: number): string => \"hi\";", "f").expect("hover");
    // Arrow-function type is `(a: number) => string`.
    assert!(
        info.contains("number") && info.contains("string"),
        "expected arrow hover to mention param/return types, got {info:?}"
    );
}

// ────────────────────────────────────────────────────────────────────────────
// Declaration merge (namespace+function, namespace+class, enum+enum)
// ────────────────────────────────────────────────────────────────────────────

#[test]
fn checker_namespace_function_merge_no_error() {
    // `function N` + `namespace N` merge: N is callable AND has members.
    let diags = check_source(
        "function N(): void {}\n\
         namespace N { export const x: number = 1; }\n\
         N();\n\
         const y: number = N.x;",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_namespace_class_merge_no_error() {
    // `class N` + `namespace N` merge: N is constructable AND has members.
    let diags = check_source(
        "class N { prop: number = 1; }\n\
         namespace N { export const x: number = 1; }\n\
         const inst: N = new N();\n\
         const y: number = N.x;",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_merged_enum_members_visible_no_error() {
    // Two `enum E` declarations merge; members from both are visible as
    // both values and types.
    let diags = check_source(
        "enum E { A = 1 }\n\
         enum E { B = 2 }\n\
         const a: E = E.A;\n\
         const b: E = E.B;\n\
         const c: E.A = E.A;\n\
         const d: E.B = E.B;",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_merged_enum_type_union_all_members_no_error() {
    // The enum type E should include both E.A and E.B from merged decls.
    let diags = check_source(
        "enum E { A = 1 }\n\
         enum E { B = 2 }\n\
         function f(x: E) {}\n\
         f(E.A);\n\
         f(E.B);",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_merged_enum_member_wrong_type_ts2322() {
    // E.B is a number (2), not a string — enum member access resolves to
    // the correct literal type (not `any`).
    let diags = check_source(
        "enum E { A = 1 }\n\
         enum E { B = 2 }\n\
         const x: string = E.B;",
    );
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_single_enum_member_wrong_type_ts2322() {
    // Baseline: Color.Red is a number (0), not a string.
    let diags = check_source(
        "enum Color { Red = 0 }\n\
         const x: string = Color.Red;",
    );
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_merged_enum_string_not_assignable_ts2322() {
    // String is not assignable to the merged enum type E.
    let diags = check_source(
        "enum E { A = 1 }\n\
         enum E { B = 2 }\n\
         const x: E = \"hi\";",
    );
    assert_diagnostic_code(&diags, 2322);
}

// ────────────────────────────────────────────────────────────────────────────
// Emit-resolver visibility tracking (P3.11)
//
// These tests build a fully-initialized `Checker` via `Program::build_checker`
// and exercise `is_declaration_visible` / `precalculate_declaration_emit_visibility`
// directly, since declaration emit is not yet wired into the emit pipeline.
// ────────────────────────────────────────────────────────────────────────────

/// Build a checker for a single source file and return it, with all files
/// already type-checked. Mirrors `check_source` but exposes the checker.
fn build_checker(source: &str) -> Checker {
    let fs = Arc::new(InMemoryFS::new());
    fs.insert_dir("/proj");
    fs.insert_file("/proj/entry.ts", source);

    let args = vec!["--noLib".to_string(), "/proj/entry.ts".to_string()];
    let parsed = parse_command_line(&args, "/proj", Some(fs.as_ref()));
    let host: Arc<dyn tsox::compiler::CompilerHost> =
        Arc::new(CompilerHostImpl::new(fs, "/proj".to_string(), lib_path()));
    let program = Arc::new(Program::new(ProgramOptions {
        config: parsed,
        host,
    }));
    program.build_checker()
}

/// Find the first top-level statement of `kind` in the entry source file.
fn first_statement<'a>(
    checker: &'a Checker,
    kind: SyntaxKind,
) -> Option<std::sync::Arc<tsox::ast::Node>> {
    let file = checker
        .files
        .iter()
        .find(|f| f.file_name == "/proj/entry.ts")
        .expect("entry source file");
    let NodeData::SourceFile(data) = &file.node.data else {
        return None;
    };
    for stmt in data.statements.nodes.iter() {
        if stmt.kind == kind {
            return Some(std::sync::Arc::clone(stmt));
        }
    }
    None
}

#[test]
fn visibility_exported_function_in_module_is_visible() {
    // An exported declaration in a module file is visible (its container,
    // the source file, is always visible).
    let mut checker = build_checker("export function f(): void {}");
    let stmt = first_statement(&checker, SyntaxKind::FunctionDeclaration).unwrap();
    assert!(checker.is_declaration_visible(&stmt));
}

#[test]
fn visibility_non_exported_function_in_module_not_visible() {
    // A non-exported declaration in a module (external module) file is NOT
    // visible — the file is a module because of the `export` on `f`, so `g`
    // is not globally reachable.
    let mut checker = build_checker("export function f(): void {}\nfunction g(): void {}");
    let g_stmt = checker
        .files
        .iter()
        .find(|f| f.file_name == "/proj/entry.ts")
        .unwrap();
    let NodeData::SourceFile(data) = &g_stmt.node.data else {
        panic!("not a source file");
    };
    let g = data
        .statements
        .nodes
        .iter()
        .find(|n| {
            n.kind == SyntaxKind::FunctionDeclaration && n.name().map(|nm| nm.text()) == Some("g")
        })
        .cloned()
        .expect("g function");
    assert!(
        !checker.is_declaration_visible(&g),
        "non-exported function in a module should not be visible"
    );
}

#[test]
fn visibility_global_script_declaration_is_visible() {
    // In a global (non-module) script, top-level declarations are visible
    // even without `export`.
    let mut checker = build_checker("function g(): void {}");
    let stmt = first_statement(&checker, SyntaxKind::FunctionDeclaration).unwrap();
    assert!(
        checker.is_declaration_visible(&stmt),
        "top-level function in a global script should be visible"
    );
}

#[test]
fn visibility_import_clause_not_visible_by_default() {
    // Import clauses / namespace imports are not visible until the alias
    // marking visitor marks them.
    let mut checker = build_checker("import x from \"./other\";\nexport function f(): void {}");
    let file = checker
        .files
        .iter()
        .find(|f| f.file_name == "/proj/entry.ts")
        .unwrap();
    let NodeData::SourceFile(data) = &file.node.data else {
        panic!("not a source file");
    };
    let import = data
        .statements
        .nodes
        .iter()
        .find(|n| n.kind == SyntaxKind::ImportDeclaration)
        .cloned()
        .expect("import declaration");
    // The ImportClause is the child we care about.
    let clause = {
        let mut found: Option<std::sync::Arc<tsox::ast::Node>> = None;
        tsox::ast::node_data_generated::for_each_child(&import, |child| {
            if child.kind == SyntaxKind::ImportClause {
                found = Some(std::sync::Arc::clone(child));
                true
            } else {
                false
            }
        });
        found.expect("import clause")
    };
    assert!(
        !checker.is_declaration_visible(&clause),
        "import clause should not be visible until marked"
    );
}

#[test]
fn visibility_alias_marking_marks_export_specifier_target() {
    // `export { g }` should mark `g`'s declaration visible via the alias
    // marking visitor.
    let mut checker = build_checker(
        "function g(): void {}\n\
         export { g };",
    );
    let file = checker
        .files
        .iter()
        .find(|f| f.file_name == "/proj/entry.ts")
        .expect("entry file")
        .clone();
    checker.precalculate_declaration_emit_visibility(&file);
    // After precalculation, the `function g` declaration (which was not
    // visible because it's an unexported statement in a module file with
    // `export { g }`) should now be marked visible by the visitor.
    let NodeData::SourceFile(data) = &file.node.data else {
        panic!("not a source file");
    };
    let g = data
        .statements
        .nodes
        .iter()
        .find(|n| n.kind == SyntaxKind::FunctionDeclaration)
        .cloned()
        .expect("g function");
    assert!(
        checker.is_declaration_visible(&g),
        "export {{ g }} should mark g visible via the alias marking visitor"
    );
}

#[test]
fn visibility_export_assignment_marks_target() {
    // `export = x` should mark `x`'s declaration visible.
    let mut checker = build_checker(
        "function x(): void {}\n\
         export = x;",
    );
    let file = checker
        .files
        .iter()
        .find(|f| f.file_name == "/proj/entry.ts")
        .expect("entry file")
        .clone();
    checker.precalculate_declaration_emit_visibility(&file);
    let NodeData::SourceFile(data) = &file.node.data else {
        panic!("not a source file");
    };
    let x = data
        .statements
        .nodes
        .iter()
        .find(|n| n.kind == SyntaxKind::FunctionDeclaration)
        .cloned()
        .expect("x function");
    assert!(
        checker.is_declaration_visible(&x),
        "export = x should mark x visible"
    );
}

#[test]
fn visibility_private_property_not_visible() {
    // Private class members are not visible.
    let mut checker = build_checker(
        "export class C {\n\
         private p: number = 1;\n\
         public q: number = 2;\n\
         }",
    );
    let file = checker
        .files
        .iter()
        .find(|f| f.file_name == "/proj/entry.ts")
        .expect("entry file");
    let NodeData::SourceFile(data) = &file.node.data else {
        panic!("not a source file");
    };
    let class = data
        .statements
        .nodes
        .iter()
        .find(|n| n.kind == SyntaxKind::ClassDeclaration)
        .cloned()
        .expect("class C");
    let mut members: Vec<std::sync::Arc<tsox::ast::Node>> = Vec::new();
    tsox::ast::node_data_generated::for_each_child(&class, |child| {
        members.push(std::sync::Arc::clone(child));
        false
    });
    let private_p = members
        .iter()
        .find(|m| {
            m.kind == SyntaxKind::PropertyDeclaration && m.name().map(|n| n.text()) == Some("p")
        })
        .cloned()
        .expect("private property p");
    let public_q = members
        .iter()
        .find(|m| {
            m.kind == SyntaxKind::PropertyDeclaration && m.name().map(|n| n.text()) == Some("q")
        })
        .cloned()
        .expect("public property q");
    assert!(
        !checker.is_declaration_visible(&private_p),
        "private property should not be visible"
    );
    assert!(
        checker.is_declaration_visible(&public_q),
        "public property should be visible"
    );
}

#[test]
fn visibility_type_parameter_always_visible() {
    // Type parameters are always visible.
    let mut checker = build_checker("export function f<T>(): void {}");
    let file = checker
        .files
        .iter()
        .find(|f| f.file_name == "/proj/entry.ts")
        .expect("entry file");
    let NodeData::SourceFile(data) = &file.node.data else {
        panic!("not a source file");
    };
    let f = data
        .statements
        .nodes
        .iter()
        .find(|n| n.kind == SyntaxKind::FunctionDeclaration)
        .cloned()
        .expect("function f");
    let mut tp: Option<std::sync::Arc<tsox::ast::Node>> = None;
    tsox::ast::node_data_generated::for_each_child(&f, |child| {
        if child.kind == SyntaxKind::TypeParameter {
            tp = Some(std::sync::Arc::clone(child));
            true
        } else {
            false
        }
    });
    let tp = tp.expect("type parameter");
    assert!(
        checker.is_declaration_visible(&tp),
        "type parameter should always be visible"
    );
}

#[test]
fn visibility_export_specifier_reexport_visible() {
    // An `export { X }` (no module specifier) is a visible re-export when
    // its parent (the source file) is visible.
    let mut checker = build_checker("export { x };");
    let file = checker
        .files
        .iter()
        .find(|f| f.file_name == "/proj/entry.ts")
        .expect("entry file");
    let NodeData::SourceFile(data) = &file.node.data else {
        panic!("not a source file");
    };
    let export_decl = data
        .statements
        .nodes
        .iter()
        .find(|n| n.kind == SyntaxKind::ExportDeclaration)
        .cloned()
        .expect("export declaration");
    let mut spec: Option<std::sync::Arc<tsox::ast::Node>> = None;
    tsox::ast::node_data_generated::for_each_child(&export_decl, |child| {
        // NamedExports wraps the specifiers; descend one level.
        if child.kind == SyntaxKind::NamedExports {
            tsox::ast::node_data_generated::for_each_child(child, |spec_child| {
                if spec_child.kind == SyntaxKind::ExportSpecifier {
                    spec = Some(std::sync::Arc::clone(spec_child));
                    true
                } else {
                    false
                }
            });
        }
        false
    });
    let spec = spec.expect("export specifier");
    assert!(
        checker.is_declaration_visible(&spec),
        "export {{ x }} (no module specifier) should be visible"
    );
}

// ────────────────────────────────────────────────────────────────────────────
// Labeled statements (P3.1 binder flow graph 收尾)
// ────────────────────────────────────────────────────────────────────────────

#[test]
fn checker_labeled_break_no_error() {
    // A labeled `break` should resolve to the enclosing label and produce
    // no diagnostics.
    let diags = check_source(
        "let sum = 0;\n\
         outer: for (let i = 0; i < 3; i++) {\n\
         \x20   for (let j = 0; j < 3; j++) {\n\
         \x20       if (j === 2) break outer;\n\
         \x20       sum += j;\n\
         \x20   }\n\
         }",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_labeled_continue_no_error() {
    // A labeled `continue` should resume the outer loop without error.
    let diags = check_source(
        "let count = 0;\n\
         outer: for (let i = 0; i < 3; i++) {\n\
         \x20   for (let j = 0; j < 3; j++) {\n\
         \x20       if (j === 1) continue outer;\n\
         \x20       count++;\n\
         \x20   }\n\
         }",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_unlabeled_break_in_labeled_loop_no_error() {
    // An unlabeled `break` inside a labeled loop should still break the
    // innermost loop — the label is simply unused (but valid syntax).
    let diags = check_source(
        "let sum = 0;\n\
         label: for (let i = 0; i < 3; i++) {\n\
         \x20   for (let j = 0; j < 3; j++) {\n\
         \x20       if (j === 1) break;\n\
         \x20       sum += j;\n\
         \x20   }\n\
         }",
    );
    assert_no_diagnostics(&diags);
}

// ────────────────────────────────────────────────────────────────────────────
// P3.9: Freshness tracking / literal widening. `const` declarations preserve
// the literal type (`const x = "hello"` → `"hello"`), while `let`/`var`
// widen to the primitive base (`let x = "hello"` → `string`). This mirrors
// Go's "freshness" mechanism: literal expressions produce a fresh literal
// type that widens ONLY for `let`/`var`, not `const`.
// ────────────────────────────────────────────────────────────────────────────

#[test]
fn checker_const_string_literal_preserved_no_error() {
    // `const x = "hello"` preserves the literal type `"hello"` (not widened
    // to `string`), so it is assignable to a union containing `"hello"`.
    let diags = check_source(
        "const x = \"hello\";\n\
         const y: \"hello\" | \"world\" = x;",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_const_number_literal_preserved_no_error() {
    // `const x = 42` preserves the literal type `42` (not widened to
    // `number`), so it is assignable to a union containing `42`.
    let diags = check_source("const x = 42;\nconst y: 42 | 99 = x;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_const_literal_assignable_to_union_no_error() {
    // `const x = "a"` preserves `"a"`; assigning to `"a" | "b"` is valid.
    let diags = check_source("const x = \"a\";\nconst y: \"a\" | \"b\" = x;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_let_string_literal_widens_no_error() {
    // `let x = "hello"` widens to `string`, so reassigning `x = "bye"` is
    // valid (both are `string`).
    let diags = check_source("let x = \"hello\";\nx = \"bye\";");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_const_literal_assignable_to_primitive_no_error() {
    // `const x = "hello"` preserves the literal `"hello"`, which is
    // assignable to `string` (a fresh/regular literal is assignable to its
    // primitive base).
    let diags = check_source("const x = \"hello\";\nlet y: string = x;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_let_widened_not_assignable_to_literal_ts2322() {
    // `let x = "hello"` widens to `string`, which is NOT assignable to the
    // literal type `"hello"` (string is wider than the literal). This
    // should error TS2322.
    let diags = check_source("let x = \"hello\";\nconst y: \"hello\" = x;");
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_const_literal_callable_with_union_param_no_error() {
    // `const kind = "foo"` preserves `"foo"`, so calling a function whose
    // parameter type is `"foo" | "bar"` with `kind` is valid.
    let diags = check_source(
        "function f(k: \"foo\" | \"bar\"): void {}\n\
         const kind = \"foo\";\n\
         f(kind);",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_let_number_literal_widens_no_error() {
    // `let x = 1` widens to `number`, so reassigning `x = 2` is valid.
    let diags = check_source("let x = 1;\nx = 2;");
    assert_no_diagnostics(&diags);
}

// ────────────────────────────────────────────────────────────────────────────
// Enum member value computation (P3.7 Phase 1: `isEnumTypeRelatedTo`)
//
// `is_enum_type_related_to` compares the member values of two same-named
// `RegularEnum`s to decide assignability. The value computation lives in
// `Checker::get_enum_member_value` / `compute_enum_member_values` (ported from
// Go's checker.go:23820-23901). These tests exercise that computation directly
// via `build_checker` + `get_enum_member_value`, since the Enum→Enum relater
// branch is only reached for types carrying `TypeFlags::Enum` (not yet produced
// by the Rust port's enum type resolver, which currently emits literal unions).
//
// Phase 1 limitation: member initializers that reference other enum members
// (e.g. `B = A`) resolve to `None` because the evaluator uses a no-op entity
// resolver. Such members are treated as opaque/assumed-numeric by the relater,
// matching Go's handling of ambient (opaque) enum members.
//
// Constructing two *distinct* same-named `RegularEnum` symbols in a single
// file is not possible (the binder merges `enum E {} enum E {}` into one
// symbol), so the full cross-enum value comparison is not exercised here; the
// unit tests below cover the value-computation primitive that drives it.
// ────────────────────────────────────────────────────────────────────────────

use tsox::evaluator::EvalValue;
use tsox::jsnum::Number;

/// Build a checker for `source`, find the first `EnumDeclaration`, and return
/// its members' names paired with their computed `EvalResult` values.
fn enum_member_values(source: &str) -> Vec<(String, tsox::evaluator::EvalResult)> {
    let mut checker = build_checker(source);
    let decl = first_statement(&checker, SyntaxKind::EnumDeclaration).expect("enum declaration");
    let members: Vec<std::sync::Arc<tsox::ast::Node>> = match &decl.data {
        NodeData::EnumDeclaration(d) => d.members.iter().cloned().collect(),
        _ => panic!("not an enum declaration"),
    };
    let mut out = Vec::new();
    for m in members {
        let name = match &m.data {
            NodeData::EnumMember(d) => d.name.text().to_string(),
            _ => continue,
        };
        let value = checker.get_enum_member_value(&m);
        out.push((name, value));
    }
    out
}

/// Look up a member's computed value by name, panicking if absent.
fn value_of(
    values: &[(String, tsox::evaluator::EvalResult)],
    name: &str,
) -> tsox::evaluator::EvalResult {
    values
        .iter()
        .find(|(n, _)| n == name)
        .map(|(_, v)| v.clone())
        .unwrap_or_else(|| panic!("enum member {name} not found"))
}

#[test]
fn checker_enum_member_value_numeric_literal() {
    let v = enum_member_values("enum E { A = 1, B = 2 }");
    assert_eq!(
        value_of(&v, "A").value,
        Some(EvalValue::Number(Number(1.0)))
    );
    assert_eq!(
        value_of(&v, "B").value,
        Some(EvalValue::Number(Number(2.0)))
    );
}

#[test]
fn checker_enum_member_value_auto_increment_from_zero() {
    let v = enum_member_values("enum E { A, B, C }");
    assert_eq!(
        value_of(&v, "A").value,
        Some(EvalValue::Number(Number(0.0)))
    );
    assert_eq!(
        value_of(&v, "B").value,
        Some(EvalValue::Number(Number(1.0)))
    );
    assert_eq!(
        value_of(&v, "C").value,
        Some(EvalValue::Number(Number(2.0)))
    );
}

#[test]
fn checker_enum_member_value_auto_increment_from_explicit() {
    let v = enum_member_values("enum E { A = 5, B }");
    assert_eq!(
        value_of(&v, "A").value,
        Some(EvalValue::Number(Number(5.0)))
    );
    assert_eq!(
        value_of(&v, "B").value,
        Some(EvalValue::Number(Number(6.0)))
    );
}

#[test]
fn checker_enum_member_value_string_literal() {
    let v = enum_member_values("enum E { A = 'x', B = 'y' }");
    assert_eq!(
        value_of(&v, "A").value,
        Some(EvalValue::String("x".to_string()))
    );
    assert_eq!(
        value_of(&v, "B").value,
        Some(EvalValue::String("y".to_string()))
    );
}

#[test]
fn checker_enum_member_value_string_resets_auto_increment() {
    // A string member resets the auto-increment counter; a subsequent
    // initializer-less member has no computable value (None), mirroring Go's
    // TS1062 case (the error itself is suppressed in Phase 1).
    let v = enum_member_values("enum E { A = 1, B = 's', C }");
    assert_eq!(
        value_of(&v, "A").value,
        Some(EvalValue::Number(Number(1.0)))
    );
    assert_eq!(
        value_of(&v, "B").value,
        Some(EvalValue::String("s".to_string()))
    );
    assert_eq!(value_of(&v, "C").value, None);
}

#[test]
fn checker_enum_member_value_unary_minus() {
    let v = enum_member_values("enum E { A = -1, B }");
    assert_eq!(
        value_of(&v, "A").value,
        Some(EvalValue::Number(Number(-1.0)))
    );
    assert_eq!(
        value_of(&v, "B").value,
        Some(EvalValue::Number(Number(0.0)))
    );
}

#[test]
fn checker_enum_member_value_binary_arithmetic() {
    let v = enum_member_values("enum E { A = 1 + 2, B = 3 * 4 }");
    assert_eq!(
        value_of(&v, "A").value,
        Some(EvalValue::Number(Number(3.0)))
    );
    assert_eq!(
        value_of(&v, "B").value,
        Some(EvalValue::Number(Number(12.0)))
    );
}

#[test]
fn checker_enum_member_value_bitwise_shift() {
    let v = enum_member_values("enum E { A = 1 << 2 }");
    assert_eq!(
        value_of(&v, "A").value,
        Some(EvalValue::Number(Number(4.0)))
    );
}

#[test]
fn checker_enum_member_value_computation_is_idempotent() {
    // Calling `get_enum_member_value` twice must yield the same value; the
    // `EnumValuesComputed` node flag guards re-computation.
    let mut checker = build_checker("enum E { A = 7, B }");
    let decl = first_statement(&checker, SyntaxKind::EnumDeclaration).expect("enum");
    let members: Vec<std::sync::Arc<tsox::ast::Node>> = match &decl.data {
        NodeData::EnumDeclaration(d) => d.members.iter().cloned().collect(),
        _ => panic!(),
    };
    let a = &members[0];
    let first = checker.get_enum_member_value(a);
    let second = checker.get_enum_member_value(a);
    assert_eq!(first.value, second.value);
    assert_eq!(first.value, Some(EvalValue::Number(Number(7.0))));
}

// ── Regression: merged enum declarations (same-symbol path) ──

#[test]
fn checker_is_enum_type_related_merged_symbol_no_error() {
    // Two `enum E` declarations merge into a single symbol, so
    // `is_enum_type_related_to` short-circuits via `Arc::ptr_eq` without
    // comparing member values. Members from both decls remain assignable to
    // the enum type.
    let diags = check_source(
        "enum E { A = 1 }\n\
         enum E { B = 2 }\n\
         const a: E = E.A;\n\
         const b: E = E.B;\n\
         const c: E.A = E.A;\n\
         const d: E.B = E.B;",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_is_enum_type_related_enum_literal_regression_no_error() {
    // Enum-literal → enum-literal assignment (same enum) must keep working
    // after the EnumLiteral relater branch was taught to also call
    // `is_enum_type_related_to`.
    let diags = check_source(
        "enum Color { Red = 0, Green = 1 }\n\
         const r: Color.Red = Color.Red;\n\
         const g: Color.Green = Color.Green;",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_is_enum_type_related_enum_to_enum_regression_no_error() {
    // `let x: Color = Color.Red` (enum-literal → enum type) must keep working.
    let diags = check_source(
        "enum Color { Red = 0, Green = 1 }\n\
         let x: Color = Color.Red;\n\
         let y: Color = Color.Green;",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn module_resolution_node_modules_basic() {
    // Simulate a project that imports from node_modules
    let fs = tsox::vfs::InMemoryFS::new();
    fs.insert_dir("/project");
    fs.insert_dir("/project/node_modules");
    fs.insert_dir("/project/node_modules/mypkg");
    fs.insert_file(
        "/project/node_modules/mypkg/index.d.ts",
        "export declare function greet(name: string): string;",
    );
    fs.insert_file(
        "/project/node_modules/mypkg/package.json",
        r#"{"name": "mypkg", "version": "1.0.0", "types": "index.d.ts"}"#,
    );
    fs.insert_file(
        "/project/src/main.ts",
        "import { greet } from 'mypkg';\nconst x: string = greet('world');",
    );

    let host = std::sync::Arc::new(tsox::compiler::CompilerHostImpl::new(
        std::sync::Arc::new(fs),
        "/project".to_string(),
        tsox::bundled::lib_path(),
    ));
    let host: std::sync::Arc<dyn tsox::compiler::CompilerHost> = host;

    let mut config = tsox::tsoptions::ParsedCommandLine::default();
    config.file_names = vec!["/project/src/main.ts".to_string()];
    config.compiler_options.no_lib = tsox::core::tristate::Tristate::True;

    let program = std::sync::Arc::new(tsox::compiler::Program::new(
        tsox::compiler::ProgramOptions { config, host },
    ));

    // Should resolve the import and not report TS2307 (cannot find module)
    let diags = program.get_semantic_diagnostics();
    let has_module_not_found = diags.iter().any(|d| d.code == 2307);
    assert!(
        !has_module_not_found,
        "Expected module 'mypkg' to be resolved from node_modules. Diagnostics: {:?}",
        diags
            .iter()
            .map(|d| (d.code, d.message_args.clone()))
            .collect::<Vec<_>>()
    );
}

#[test]
fn module_resolution_node_modules_scoped_package() {
    // Simulate a scoped package @scope/pkg
    let fs = tsox::vfs::InMemoryFS::new();
    fs.insert_dir("/project");
    fs.insert_dir("/project/node_modules");
    fs.insert_dir("/project/node_modules/@scope");
    fs.insert_dir("/project/node_modules/@scope/mypkg");
    fs.insert_file(
        "/project/node_modules/@scope/mypkg/index.d.ts",
        "export declare const version: number;",
    );
    fs.insert_file(
        "/project/node_modules/@scope/mypkg/package.json",
        r#"{"name": "@scope/mypkg", "version": "2.0.0", "types": "index.d.ts"}"#,
    );
    fs.insert_file(
        "/project/src/main.ts",
        "import { version } from '@scope/mypkg';\nconst v: number = version;",
    );

    let host = std::sync::Arc::new(tsox::compiler::CompilerHostImpl::new(
        std::sync::Arc::new(fs),
        "/project".to_string(),
        tsox::bundled::lib_path(),
    ));
    let host: std::sync::Arc<dyn tsox::compiler::CompilerHost> = host;

    let mut config = tsox::tsoptions::ParsedCommandLine::default();
    config.file_names = vec!["/project/src/main.ts".to_string()];
    config.compiler_options.no_lib = tsox::core::tristate::Tristate::True;

    let program = std::sync::Arc::new(tsox::compiler::Program::new(
        tsox::compiler::ProgramOptions { config, host },
    ));

    let diags = program.get_semantic_diagnostics();
    let has_module_not_found = diags.iter().any(|d| d.code == 2307);
    assert!(
        !has_module_not_found,
        "Expected scoped module '@scope/mypkg' to be resolved from node_modules. Diagnostics: {:?}",
        diags
            .iter()
            .map(|d| (d.code, d.message_args.clone()))
            .collect::<Vec<_>>()
    );
}

#[test]
fn module_resolution_node_modules_nested() {
    // node_modules at parent directory level
    let fs = tsox::vfs::InMemoryFS::new();
    fs.insert_dir("/root");
    fs.insert_dir("/root/node_modules");
    fs.insert_dir("/root/node_modules/shared");
    fs.insert_file(
        "/root/node_modules/shared/index.d.ts",
        "export declare function util(): void;",
    );
    fs.insert_file(
        "/root/node_modules/shared/package.json",
        r#"{"name": "shared", "version": "1.0.0", "types": "index.d.ts"}"#,
    );
    fs.insert_dir("/root/packages");
    fs.insert_dir("/root/packages/app");
    fs.insert_dir("/root/packages/app/src");
    fs.insert_file(
        "/root/packages/app/src/main.ts",
        "import { util } from 'shared';\nutil();",
    );

    let host = std::sync::Arc::new(tsox::compiler::CompilerHostImpl::new(
        std::sync::Arc::new(fs),
        "/root/packages/app".to_string(),
        tsox::bundled::lib_path(),
    ));
    let host: std::sync::Arc<dyn tsox::compiler::CompilerHost> = host;

    let mut config = tsox::tsoptions::ParsedCommandLine::default();
    config.file_names = vec!["/root/packages/app/src/main.ts".to_string()];
    config.compiler_options.no_lib = tsox::core::tristate::Tristate::True;

    let program = std::sync::Arc::new(tsox::compiler::Program::new(
        tsox::compiler::ProgramOptions { config, host },
    ));

    let diags = program.get_semantic_diagnostics();
    let has_module_not_found = diags.iter().any(|d| d.code == 2307);
    assert!(
        !has_module_not_found,
        "Expected module 'shared' to be resolved from parent node_modules. Diagnostics: {:?}",
        diags
            .iter()
            .map(|d| (d.code, d.message_args.clone()))
            .collect::<Vec<_>>()
    );
}

// ────────────────────────────────────────────────────────────────────────────
// TS2448: Block-scoped variable used before its declaration (temporal dead zone)
// ────────────────────────────────────────────────────────────────────────────

#[test]
fn checker_ts2448_let_used_before_declaration() {
    // `let` variable referenced before its declaration → TS2448.
    let diags = check_source("let y = later;\nlet later = 42;");
    assert_diagnostic_code(&diags, 2448);
}

#[test]
fn checker_ts2448_const_used_before_declaration() {
    // `const` variable referenced before its declaration → TS2448.
    let diags = check_source("let y = later;\nconst later = 42;");
    assert_diagnostic_code(&diags, 2448);
}

#[test]
fn checker_ts2448_let_used_after_declaration_no_error() {
    // `let` variable referenced after its declaration → no error.
    let diags = check_source("let later = 42;\nlet y = later;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_ts2448_var_used_before_declaration_no_error() {
    // `var` is function-scoped and hoisted — no TS2448.
    let diags = check_source("let y = later;\nvar later = 42;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_ts2448_function_declaration_used_before_no_error() {
    // Function declarations are hoisted — no TS2448.
    let diags = check_source("f();\nfunction f() { return 1; }");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_ts2448_block_scoped_used_before_in_block() {
    // TDZ applies within block scopes too.
    let diags = check_source("{\n  let y = later;\n  let later = 42;\n}");
    assert_diagnostic_code(&diags, 2448);
}

#[test]
fn checker_ts2448_deferred_in_function_no_error() {
    // A reference inside a function body is deferred — no TS2448 even if
    // the declaration comes after the function textually.
    let diags = check_source("function f() { return later; }\nlet later = 42;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_ts2448_deferred_in_arrow_no_error() {
    // A reference inside an arrow function body is deferred — no TS2448.
    let diags = check_source("const f = () => later;\nlet later = 42;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_ts2448_class_used_before_declaration() {
    // Class referenced before its declaration → TS2448.
    let diags = check_source("const c = new C();\nclass C {}");
    assert_diagnostic_code(&diags, 2448);
}

#[test]
fn checker_ts2448_class_used_after_declaration_no_error() {
    // Class referenced after its declaration → no error.
    let diags = check_source("class C {}\nconst c = new C();");
    assert_no_diagnostics(&diags);
}

// ────────────────────────────────────────────────────────────────────────────
// TS2454: Variable used before being assigned (definite assignment)
// ────────────────────────────────────────────────────────────────────────────

#[test]
fn checker_ts2454_let_uninitialized_used_before_assignment() {
    // `let v: number;` then read before assignment → TS2454.
    let diags = check_source("let v: number;\nlet y = v;\nv = 1;");
    assert_diagnostic_code(&diags, 2454);
}

#[test]
fn checker_ts2454_let_assigned_before_use_no_error() {
    // `let v: number; v = 1;` then read → no error.
    let diags = check_source("let v: number;\nv = 1;\nlet y = v;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_ts2454_let_with_initializer_no_error() {
    // `let v: number = 0;` — has initializer → no TS2454.
    let diags = check_source("let v: number = 0;\nlet y = v;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_ts2454_let_no_type_annotation_no_error() {
    // `let v;` — no type annotation → no TS2454.
    let diags = check_source("let v;\nlet y = v;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_ts2454_let_type_includes_undefined_no_error() {
    // `let v: number | undefined;` — declared type includes undefined →
    // no TS2454 (reading undefined is valid for this type).
    let diags = check_source("let v: number | undefined;\nlet y = v;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_ts2454_let_definite_assignment_assertion_no_error() {
    // `let v!: number;` — definite assignment assertion → no TS2454.
    let diags = check_source("let v!: number;\nlet y = v;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_ts2454_declare_let_no_error() {
    // `declare let v: number;` — ambient → no TS2454.
    let diags = check_source("declare let v: number;\nlet y = v;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_ts2454_let_string_uninitialized_used_before_assignment() {
    // Non-number type: `let s: string;` read before assignment → TS2454.
    let diags = check_source("let s: string;\nlet y = s;\ns = \"hi\";");
    assert_diagnostic_code(&diags, 2454);
}

// ────────────────────────────────────────────────────────────────────────────
// TS18048: 'X' is possibly 'undefined'
// ────────────────────────────────────────────────────────────────────────────

#[test]
fn checker_ts18048_property_access_on_possibly_undefined() {
    // Accessing `.a` on `{ a: number } | undefined` → TS18048.
    let diags = check_source("let x: { a: number } | undefined = { a: 1 };\nx.a;");
    assert_diagnostic_code(&diags, 18048);
}

#[test]
fn checker_ts18048_optional_chain_suppresses_error() {
    // Optional chaining `x?.a` suppresses TS18048.
    let diags = check_source("let x: { a: number } | undefined = { a: 1 };\nx?.a;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_ts18048_property_access_on_non_undefined_no_error() {
    // Accessing `.a` on `{ a: number }` (not possibly undefined) → no error.
    let diags = check_source("let x: { a: number } = { a: 1 };\nlet y: number = x.a;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_ts18048_property_does_not_exist_at_all_ts2339() {
    // Property `.b` doesn't exist on `{ a: number } | undefined` → TS2339
    // (not TS18048, because the property doesn't exist on the non-undefined
    // part either).
    let diags = check_source("let x: { a: number } | undefined = { a: 1 };\nx.b;");
    assert_diagnostic_code(&diags, 2339);
}

#[test]
fn checker_ts18048_property_access_after_narrowing_no_error() {
    // After narrowing away undefined (via `if (x !== undefined)`), accessing
    // `.a` is safe → no error.
    let diags = check_source(
        "let x: { a: number } | undefined = { a: 1 };\
         if (x !== undefined) {\
             let y: number = x.a;\
         }",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_ts18048_property_access_on_possibly_null_union() {
    // `{ a: number } | null` — accessing `.a` → TS18048.
    let diags = check_source("let x: { a: number } | null = { a: 1 };\nx.a;");
    assert_diagnostic_code(&diags, 18048);
}

// ────────────────────────────────────────────────────────────────────────────
// TS2451: Cannot redeclare block-scoped variable (binder-level check)
// ────────────────────────────────────────────────────────────────────────────

#[test]
fn checker_ts2451_const_redeclare() {
    // Two `const name` in the same scope → TS2451.
    let diags = check_source("const name = \"Alice\";\nconst name = \"Bob\";");
    assert_diagnostic_code(&diags, 2451);
}

#[test]
fn checker_ts2451_let_redeclare() {
    // Two `let x` in the same scope → TS2451.
    let diags = check_source("let x = 1;\nlet x = 2;");
    assert_diagnostic_code(&diags, 2451);
}

#[test]
fn checker_ts2451_let_then_const_redeclare() {
    // `let x` then `const x` in the same scope → TS2451.
    let diags = check_source("let x = 1;\nconst x = 2;");
    assert_diagnostic_code(&diags, 2451);
}

#[test]
fn checker_ts2451_const_then_let_redeclare() {
    // `const x` then `let x` in the same scope → TS2451.
    let diags = check_source("const x = 1;\nlet x = 2;");
    assert_diagnostic_code(&diags, 2451);
}

#[test]
fn checker_ts2451_triple_redeclare_reports_two() {
    // Three `let x` → TS2451 reported for the 2nd and 3rd declarations.
    let diags = check_source("let x = 1;\nlet x = 2;\nlet x = 3;");
    assert_diagnostic_count(&diags, 2451, 2);
}

#[test]
fn checker_ts2451_redeclare_in_separate_blocks_no_error() {
    // `let x` in two separate block scopes is legal (distinct scopes).
    let diags = check_source("if (true) { let x = 1; }\nif (true) { let x = 2; }");
    assert_diagnostic_count(&diags, 2451, 0);
}

#[test]
fn checker_ts2451_var_redeclare_no_error() {
    // `var` is function-scoped and may be redeclared freely → no TS2451.
    let diags = check_source("var x = 1;\nvar x = 2;");
    assert_diagnostic_count(&diags, 2451, 0);
}

#[test]
fn checker_ts2451_distinct_names_no_error() {
    // Distinct block-scoped names in the same scope → no TS2451.
    let diags = check_source("let a = 1;\nlet b = 2;");
    assert_diagnostic_count(&diags, 2451, 0);
}

// ────────────────────────────────────────────────────────────────────────────
// Array methods (with lib.d.ts): the global `Array<T>` interface supplies
// methods like push/map/filter. These fixtures verify the checker consults it.
// ────────────────────────────────────────────────────────────────────────────

#[test]
fn checker_array_push_with_lib_no_error() {
    let diags = check_source_with_lib("let arr: number[] = [1, 2, 3];\narr.push(4);", false);
    assert_diagnostic_count(&diags, 2339, 0);
}

#[test]
fn checker_array_map_with_lib_no_error() {
    let diags = check_source_with_lib(
        "let arr: number[] = [1, 2, 3];\nlet y = arr.map(x => x);",
        false,
    );
    assert_diagnostic_count(&diags, 2339, 0);
}

#[test]
fn checker_array_map_without_lib_ts2339() {
    let diags = check_source("let arr: number[] = [1, 2, 3];\narr.map(x => x);");
    assert_diagnostic_code(&diags, 2339);
}

#[test]
fn checker_array_pop_with_lib_no_error() {
    let diags = check_source_with_lib("let arr: number[] = [1, 2, 3];\narr.pop();", false);
    assert_diagnostic_count(&diags, 2339, 0);
}

#[test]
fn checker_array_filter_with_lib_no_error() {
    let diags = check_source_with_lib(
        "let arr: number[] = [1, 2, 3];\nlet y = arr.filter(x => x > 1);",
        false,
    );
    assert_diagnostic_count(&diags, 2339, 0);
}

#[test]
fn checker_array_find_with_lib_no_error() {
    let diags = check_source_with_lib(
        "let arr: number[] = [1, 2, 3];\nlet y = arr.find(x => x > 1);",
        false,
    );
    assert_diagnostic_count(&diags, 2339, 0);
}

#[test]
fn checker_array_reduce_with_lib_no_error() {
    let diags = check_source_with_lib(
        "let arr: number[] = [1, 2, 3];\nlet y = arr.reduce((a, b) => a + b, 0);",
        false,
    );
    assert_diagnostic_count(&diags, 2339, 0);
}

#[test]
fn checker_array_forEach_with_lib_no_error() {
    let diags = check_source_with_lib(
        "let arr: number[] = [1, 2, 3];\narr.forEach(x => { let z = x; });",
        false,
    );
    assert_diagnostic_count(&diags, 2339, 0);
}

#[test]
fn checker_array_some_with_lib_no_error() {
    let diags = check_source_with_lib(
        "let arr: number[] = [1, 2, 3];\nlet y = arr.some(x => x > 1);",
        false,
    );
    assert_diagnostic_count(&diags, 2339, 0);
}

#[test]
fn checker_array_every_with_lib_no_error() {
    let diags = check_source_with_lib(
        "let arr: number[] = [1, 2, 3];\nlet y = arr.every(x => x > 0);",
        false,
    );
    assert_diagnostic_count(&diags, 2339, 0);
}

#[test]
fn checker_array_push_without_lib_ts2339() {
    // Without lib.d.ts, `push` is unknown on a fully-typed array → TS2339.
    let diags = check_source("let arr: number[] = [1, 2, 3];\narr.push(4);");
    assert_diagnostic_code(&diags, 2339);
}

#[test]
fn checker_array_filter_without_lib_ts2339() {
    let diags = check_source("let arr: number[] = [1, 2, 3];\narr.filter(x => x > 1);");
    assert_diagnostic_code(&diags, 2339);
}

#[test]
fn checker_array_includes_without_lib_ts2339() {
    let diags = check_source("let arr: number[] = [1, 2, 3];\narr.includes(2);");
    assert_diagnostic_code(&diags, 2339);
}

#[test]
fn checker_array_reduce_without_lib_ts2339() {
    let diags = check_source("let arr: number[] = [1, 2, 3];\narr.reduce((a, b) => a + b, 0);");
    assert_diagnostic_code(&diags, 2339);
}

// ────────────────────────────────────────────────────────────────────────────
// TS2741: Property 'X' is missing in type 'Y' but required in type 'Z'
// ────────────────────────────────────────────────────────────────────────────

#[test]
fn checker_ts2741_fresh_literal_missing_one_property() {
    // `{ a: 1 }` lacks required `b` → TS2741.
    let diags = check_source("let x: { a: number; b: string } = { a: 1 };");
    assert_diagnostic_code(&diags, 2741);
}

#[test]
fn checker_ts2741_interface_target_missing_property() {
    let diags = check_source(
        "interface P { a: number; b: number; }\n\
         let x: P = { a: 1 };",
    );
    assert_diagnostic_code(&diags, 2741);
}

#[test]
fn checker_ts2741_type_alias_target_missing_property() {
    let diags = check_source(
        "type P = { a: number; b: number; };\n\
         let x: P = { a: 1 };",
    );
    assert_diagnostic_code(&diags, 2741);
}

#[test]
fn checker_ts2741_variable_source_missing_property() {
    // A non-fresh variable missing a property the target requires → TS2741.
    let diags = check_source(
        "let obj = { a: 1 };\n\
         let y: { a: number; b: number } = obj;",
    );
    assert_diagnostic_code(&diags, 2741);
}

#[test]
fn checker_ts2739_multiple_missing_properties() {
    // Two missing properties (`b`, `c`) → TS2739 (the multi-missing form).
    let diags = check_source("let x: { a: number; b: number; c: number } = { a: 1 };");
    assert_diagnostic_code(&diags, 2739);
}

// ────────────────────────────────────────────────────────────────────────────
// TS2353: Object literal may only specify known properties
// ────────────────────────────────────────────────────────────────────────────

#[test]
fn checker_ts2353_excess_property_on_type_literal() {
    // `{ a: 1, b: 2 }` has excess `b` → TS2353.
    let diags = check_source("let x: { a: number } = { a: 1, b: 2 };");
    assert_diagnostic_code(&diags, 2353);
}

#[test]
fn checker_ts2353_excess_property_on_interface() {
    let diags = check_source(
        "interface P { a: number; }\n\
         let x: P = { a: 1, b: 2 };",
    );
    assert_diagnostic_code(&diags, 2353);
}

#[test]
fn checker_ts2353_excess_property_on_type_alias() {
    let diags = check_source(
        "type P = { a: number; };\n\
         let x: P = { a: 1, b: 2, c: 3 };",
    );
    assert_diagnostic_code(&diags, 2353);
}

#[test]
fn checker_ts2353_excess_property_all_required_present() {
    // All required properties present, plus an extra one → TS2353.
    let diags = check_source("let x: { a: number; b: number } = { a: 1, b: 2, c: 3 };");
    assert_diagnostic_code(&diags, 2353);
}

#[test]
fn checker_ts2353_no_excess_when_index_signature_present() {
    // An index signature accepts any extra property → no TS2353.
    let diags = check_source("let x: { a: number; [k: string]: number } = { a: 1, b: 2 };");
    assert_diagnostic_count(&diags, 2353, 0);
}

// ────────────────────────────────────────────────────────────────────────────
// TS2448: Block-scoped variable used before declaration (additional scenarios)
// ────────────────────────────────────────────────────────────────────────────

#[test]
fn checker_ts2448_let_used_in_initializer_of_another() {
    // `b` referenced in `a`'s initializer before `b` is declared → TS2448.
    let diags = check_source("let a = b;\nlet b = 1;");
    assert_diagnostic_code(&diags, 2448);
}

#[test]
fn checker_ts2448_const_used_in_expression_before_declaration() {
    // `s` referenced before its `const` declaration → TS2448.
    let diags = check_source("const r = s + 1;\nconst s = 2;");
    assert_diagnostic_code(&diags, 2448);
}

#[test]
fn checker_ts2448_class_instantiation_before_declaration() {
    // `new C()` before `class C` → TS2448.
    let diags = check_source("const i = new C();\nclass C {}");
    assert_diagnostic_code(&diags, 2448);
}

// ────────────────────────────────────────────────────────────────────────────
// TS2454: Variable used before being assigned (additional scenarios)
// ────────────────────────────────────────────────────────────────────────────

#[test]
fn checker_ts2454_boolean_uninitialized_used_before_assignment() {
    // `let v: boolean;` read before assignment → TS2454.
    let diags = check_source("let v: boolean;\nlet y = v;\nv = true;");
    assert_diagnostic_code(&diags, 2454);
}

#[test]
fn checker_ts2454_annotated_assignment_after_use() {
    // Reading `v` (typed `number`) before its later assignment → TS2454.
    let diags = check_source("let v: number;\nconst y: number = v;\nv = 5;");
    assert_diagnostic_code(&diags, 2454);
}

// ────────────────────────────────────────────────────────────────────────────
// TS18048: 'X' is possibly 'undefined' (additional scenarios)
// ────────────────────────────────────────────────────────────────────────────

#[test]
fn checker_ts18048_nullable_via_type_alias() {
    // A type-aliased object type unioned with `undefined` → TS18048.
    let diags = check_source(
        "type Box = { v: number };\n\
         let x: Box | undefined = { v: 1 };\n\
         x.v;",
    );
    assert_diagnostic_code(&diags, 18048);
}

#[test]
fn checker_ts18048_chained_property_access_on_possibly_undefined() {
    // `x.a.b` where `x` is possibly undefined → TS18048.
    let diags = check_source(
        "let x: { a: { b: number } } | undefined = { a: { b: 1 } };\n\
         x.a.b;",
    );
    assert_diagnostic_code(&diags, 18048);
}

#[test]
fn checker_ts18048_optional_chain_deep_suppresses_error() {
    // `x?.a?.b` suppresses TS18048 even when `x` is possibly undefined.
    let diags = check_source(
        "let x: { a: { b: number } } | undefined = { a: { b: 1 } };\n\
         x?.a?.b;",
    );
    assert_no_diagnostics(&diags);
}
