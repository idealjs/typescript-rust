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
    let fs = Arc::new(InMemoryFS::new());
    fs.insert_dir("/proj");
    fs.insert_file("/proj/entry.ts", source);

    let mut args = vec!["/proj/entry.ts".to_string()];
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

fn check_sources_with_lib(
    files: &[(&str, &str)],
    no_lib: bool,
) -> Vec<tsox::ast::Diagnostic> {
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
        panic!("Expected no diagnostics, got {}:\n{}", diags.len(), msg.join("\n"));
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
    assert_eq!(count, 2, "Expected 2 TS2304 errors for 'b' and 'c', got {}", count);
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
    let diags = check_source("function longest<T extends { length: number }>(a: T, b: T): T { return a.length >= b.length ? a : b; }");
    assert_no_diagnostics(&diags);
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
    let diags = check_source("type Named = { name: string };\ntype Aged = { age: number };\ntype Person = Named & Aged;");
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
    let diags = check_source("let a = 1; function f1() { let b = 2; function f2() { let c = 3; return a + b + c; } }");
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
    let diags = check_source("type Point = { x: number; y: number };\nlet p: Point = { x: 1, y: 2 };");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_type_alias_union_literal_no_error() {
    let diags = check_source("type Direction = 'north' | 'south' | 'east' | 'west';");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_type_alias_generic_no_error() {
    let diags = check_source("type Result<T> = { success: true; value: T } | { success: false; error: string };");
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
fn checker_class_implements_interface_no_error() {
    let diags = check_source("interface Named { name: string; }\nclass Person implements Named { name: string = 'Alice'; }");
    assert_no_diagnostics(&diags);
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
    let diags = check_source("const el = <div />;");
    // JSX identifiers are walked as children-for-expressions.
    // `div` is a tag name (not a reference), so no errors expected.
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_jsx_element_with_children_no_error() {
    let diags = check_source("const el = <div><span>hello</span></div>;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_jsx_fragment_no_error() {
    let diags = check_source("const el = <><div>a</div><div>b</div></>;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_jsx_with_expression_curly_no_error() {
    let diags = check_source("const x = 42;\nconst el = <div>{x}</div>;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_jsx_attribute_string_no_error() {
    let diags = check_source("const el = <div className='container' />;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_jsx_attribute_expression_no_error() {
    let diags = check_source("const x = 42;\nconst el = <div data-value={x} />;");
    assert_no_diagnostics(&diags);
}
#[test]
fn checker_jsx_undefined_expression_in_curly() {
    let diags = check_source("const el = <div>{undefinedVar}</div>;");
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
    assert_eq!(count, 1, "Expected 1 TS2304 error, got 0 - arrow functions have no arguments");
}

#[test]
fn checker_arguments_in_method() {
    let diags = check_source("class C { method() { return arguments; } }");
    assert_no_diagnostics(&diags);
}

// ────────────────────────────────────────────────────────────────────────────
// NameResolver: global symbol resolution
// ────────────────────────────────────────────────────────────────────────────

#[test]
fn checker_global_symbol_with_lib() {
    // With lib loaded, `Array` should be resolvable.
    let diags = check_source_with_lib("let x = Array;", false);
    let count = diags.iter().filter(|d| d.code == 2304).count();
    assert_eq!(count, 0, "Expected 0 TS2304 errors (Array is a global), got {}", count);
}

#[test]
fn checker_undefined_is_resolvable() {
    // `undefined` is a built-in global symbol.
    let diags = check_source_with_lib("let x = undefined;", false);
    let count = diags.iter().filter(|d| d.code == 2304).count();
    assert_eq!(count, 0, "Expected 0 TS2304 errors (undefined is a global), got {}", count);
}

#[test]
fn checker_global_this_is_resolvable() {
    // `globalThis` is a built-in global symbol.
    let diags = check_source_with_lib("let x = globalThis;", false);
    let count = diags.iter().filter(|d| d.code == 2304).count();
    assert_eq!(count, 0, "Expected 0 TS2304 errors (globalThis is a global), got {}", count);
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
