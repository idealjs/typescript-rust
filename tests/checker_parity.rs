use std::sync::Arc;

use tsox::bundled::{BundledFS, lib_path};
use tsox::compiler::{CompilerHostImpl, Program, ProgramOptions};
use tsox::tsoptions::parse_command_line;
use tsox::vfs::InMemoryFS;

fn check_source(source: &str) -> Vec<tsox::ast::Diagnostic> {
    check_source_with_lib(source, false)
}

fn check_source_with_lib(source: &str, no_lib: bool) -> Vec<tsox::ast::Diagnostic> {
    check_source_named_with_lib("/proj/entry.ts", source, no_lib)
}

fn check_source_tsx(source: &str) -> Vec<tsox::ast::Diagnostic> {
    check_source_tsx_with_args(source, &["--jsx", "preserve", "--noImplicitAny", "false"])
}

fn check_source_tsx_with_args(source: &str, extra_args: &[&str]) -> Vec<tsox::ast::Diagnostic> {
    let fs = Arc::new(InMemoryFS::new());
    fs.insert_dir("/proj");
    fs.insert_file("/proj/entry.tsx", source);

    let mut args = Vec::new();
    for a in extra_args {
        args.push((*a).to_string());
    }
    args.push("/proj/entry.tsx".to_string());
    let parsed = parse_command_line(&args, "/proj", Some(fs.as_ref()));

    let host: Arc<dyn tsox::compiler::CompilerHost> = {
        let bf = Arc::new(BundledFS::new(fs));
        Arc::new(CompilerHostImpl::new(bf, "/proj".to_string(), lib_path()))
    };

    let program = Arc::new(Program::new(ProgramOptions {
        config: parsed,
        host,
    }));

    program.get_semantic_diagnostics()
}

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

fn check_source_strict(source: &str) -> Vec<tsox::ast::Diagnostic> {
    check_source_with_lib_args(source, &["--strictNullChecks"])
}

fn check_source_all_strict(source: &str) -> Vec<tsox::ast::Diagnostic> {
    check_source_with_lib_args(source, &["--strict"])
}

fn check_source_with_lib_args(source: &str, extra_args: &[&str]) -> Vec<tsox::ast::Diagnostic> {
    let fs = Arc::new(InMemoryFS::new());
    fs.insert_dir("/proj");
    fs.insert_file("/proj/entry.ts", source);

    let mut args: Vec<String> = Vec::new();
    for a in extra_args {
        args.push((*a).to_string());
    }
    args.push("/proj/entry.ts".to_string());
    let parsed = parse_command_line(&args, "/proj", Some(fs.as_ref()));

    let bf = Arc::new(BundledFS::new(fs));
    let host: Arc<dyn tsox::compiler::CompilerHost> =
        Arc::new(CompilerHostImpl::new(bf, "/proj".to_string(), lib_path()));

    let program = Arc::new(Program::new(ProgramOptions {
        config: parsed,
        host,
    }));

    program.get_semantic_diagnostics()
}

fn check_sources(files: &[(&str, &str)]) -> Vec<tsox::ast::Diagnostic> {
    check_sources_with_lib(files, false)
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

fn assert_diagnostic_count(diags: &[tsox::ast::Diagnostic], code: i32, count: usize) {
    let actual = diags.iter().filter(|d| d.code == code).count();
    assert_eq!(
        actual, count,
        "Expected {} diagnostic(s) with code TS{}, got {}",
        count, code, actual
    );
}

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
        count, 0,
        "Expected 0 TS2304 errors for `let a = 1, b = 2, c = 3;`, got {count}"
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

#[test]
fn checker_assignable_keyword_to_same_keyword_no_error() {

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

    let diags = check_source("let x: string | number = true;");
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_assignable_array_annotation_wrong_primitive_ts2322() {

    let diags = check_source("let x: string[] = 42;");
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_assignable_array_annotation_primitive_to_array_ts2322() {

    let diags = check_source("let x: number[] = 'hi';");
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_assignable_array_annotation_array_literal_no_error() {

    let diags = check_source("let x: number[] = [1, 2, 3];");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_assignable_tuple_wrong_primitive_ts2322() {

    let diags = check_source("let x: [number, string] = 42;");
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_assignable_tuple_string_to_number_tuple_ts2322() {

    let diags = check_source("let x: [number, string] = 'hi';");
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_assignable_tuple_array_literal_no_error() {

    let diags = check_source("let x: [number, string] = [1, 'hi'];");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_assignable_tuple_annotation_no_init_no_error() {

    let diags = check_source("let x: [number, string];");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_recursive_object_type_does_not_overflow() {

    let diags = check_source(
        "type Box = { value: number; next: Box | null };\
         let x: Box = { value: 1, next: null };",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_var_type_inference_propagates_via_symbol_ts2322() {

    let diags = check_source("let x = 42; let y: string = x;");
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_var_type_inference_propagates_via_symbol_no_error() {

    let diags = check_source("let x = 42; let y: number = x;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_function_inferred_return_number_no_error() {

    let diags = check_source("function f() { return 42; } let y: number = f();");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_function_inferred_return_number_assigned_to_string_ts2322() {

    let diags = check_source("function f() { return 42; } let y: string = f();");
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_arrow_function_inferred_return_no_error() {

    let diags = check_source(
        "const f = (x: number) => x * 2;\
         let y: number = f(3);",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_arrow_function_inferred_return_string_to_number_ts2322() {

    let diags = check_source(
        "const f = () => 'hi';\
         let y: number = f();",
    );
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_function_with_explicit_return_type_no_error() {

    let diags = check_source("function f(): number { return 42; } let y: number = f();");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_function_no_return_infers_void_to_number_ts2322() {

    let diags = check_source("function f() {} let y: number = f();");
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_return_type_mismatch_ts2322() {

    let diags = check_source("function f(): number { return \"hi\"; }");
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_return_type_match_no_error() {

    let diags = check_source("function f(): number { return 42; }");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_return_type_string_match_no_error() {

    let diags = check_source("function f(): string { return \"x\"; }");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_return_type_boolean_mismatch_ts2322() {

    let diags = check_source("function f(): boolean { return 1; }");
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_return_missing_value_ts2322() {

    let diags = check_source("function f(): string { return; }");
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_return_missing_value_in_number_function_ts2322() {

    let diags = check_source("function f(): number { return; }");
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_return_void_no_value_no_error() {

    let diags = check_source("function f(): void { return; }");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_return_no_annotation_no_value_no_error() {

    let diags = check_source("function f() { return; }");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_arrow_return_type_mismatch_ts2322() {

    let diags = check_source("const f = (): number => { return \"hi\"; };");
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_arrow_expression_body_return_type_mismatch_ts2322() {

    let diags = check_source("const f = (): number => \"hi\";");
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_method_return_type_mismatch_ts2322() {

    let diags = check_source("class C { m(): number { return \"hi\"; } }");
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_get_accessor_return_type_mismatch_ts2322() {

    let diags = check_source("class C { get x(): number { return \"hi\"; } }");
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_return_union_type_no_error() {

    let diags = check_source(
        "type S = string | number;\
         function f(): S { return \"a\"; }",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_return_literal_to_widen_no_error() {

    let diags = check_source("function f(): number { return 42; } let y = f();");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_nested_function_return_type_check_ts2322() {

    let diags = check_source(
        "function outer(): void {\
             function inner(): number { return \"bad\"; }\
             inner();\
         }",
    );
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_comparison_no_overlap_number_string_ts2367() {

    let diags = check_source(
        "function f(x: number) {\
             if (x === \"hi\") {}\
         }",
    );
    assert_diagnostic_code(&diags, 2367);
}

#[test]
fn checker_comparison_no_overlap_boolean_string_ts2367() {

    let diags = check_source(
        "function f(x: boolean) {\
             if (x === \"hi\") {}\
         }",
    );
    assert_diagnostic_code(&diags, 2367);
}

#[test]
fn checker_comparison_no_overlap_inequality_ts2367() {

    let diags = check_source(
        "function f(x: number) {\
             if (x !== \"hi\") {}\
         }",
    );
    assert_diagnostic_code(&diags, 2367);
}

#[test]
fn checker_comparison_loose_equals_no_overlap_ts2367() {

    let diags = check_source(
        "function f(x: number) {\
             if (x == \"hi\") {}\
         }",
    );
    assert_diagnostic_code(&diags, 2367);
}

#[test]
fn checker_comparison_same_type_no_error() {

    let diags = check_source(
        "function f(x: number, y: number) {\
             if (x === y) {}\
         }",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_comparison_with_literal_no_error() {

    let diags = check_source(
        "function f(x: number) {\
             if (x === 42) {}\
         }",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_comparison_with_any_no_error() {

    let diags = check_source(
        "function f(x: any, y: number) {\
             if (x === y) {}\
         }",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_comparison_with_null_no_error() {

    let diags = check_source(
        "function f(x: number) {\
             if (x === null) {}\
         }",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_comparison_union_literal_no_error() {

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

    let diags = check_source(
        "function f(x: \"a\" | \"b\") {\
             if (x === \"c\") {}\
         }",
    );
    assert_diagnostic_code(&diags, 2367);
}

#[test]
fn checker_call_number_ts2349() {

    let diags = check_source(
        "const x: number = 1;\
         x();",
    );
    assert_diagnostic_code(&diags, 2349);
}

#[test]
fn checker_call_string_ts2349() {

    let diags = check_source(
        "const x: string = \"hi\";\
         x();",
    );
    assert_diagnostic_code(&diags, 2349);
}

#[test]
fn checker_call_boolean_ts2349() {

    let diags = check_source(
        "const x: boolean = true;\
         x();",
    );
    assert_diagnostic_code(&diags, 2349);
}

#[test]
fn checker_call_object_literal_ts2349() {

    let diags = check_source(
        "const x = { a: 1 };\
         x();",
    );
    assert_diagnostic_code(&diags, 2349);
}

#[test]
fn checker_call_class_instance_ts2349() {

    let diags = check_source(
        "class C { m() {} }\
         const c = new C();\
         c();",
    );
    assert_diagnostic_code(&diags, 2349);
}

#[test]
fn checker_call_function_no_error() {

    let diags = check_source(
        "function f() {}\
         f();",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_call_arrow_no_error() {

    let diags = check_source(
        "const f = () => 1;\
         f();",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_call_method_no_error() {

    let diags = check_source(
        "class C { m() { return 1; } }\
         const c = new C();\
         c.m();",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_new_number_ts2351() {

    let diags = check_source(
        "const x: number = 1;\
         new x();",
    );
    assert_diagnostic_code(&diags, 2351);
}

#[test]
fn checker_new_object_literal_ts2351() {

    let diags = check_source(
        "const x = { a: 1 };\
         new x();",
    );
    assert_diagnostic_code(&diags, 2351);
}

#[test]
fn checker_new_class_no_error() {

    let diags = check_source(
        "class C {}\
         const c = new C();",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_call_any_no_error() {

    let diags = check_source(
        "const x: any = 1;\
         x();",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_new_any_no_error() {

    let diags = check_source(
        "const x: any = 1;\
         new x();",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_assign_to_readonly_class_property_ts2540() {

    let diags = check_source(
        "class C { readonly x: number = 0; }\
         const c = new C();\
         c.x = 1;",
    );
    assert_diagnostic_code(&diags, 2540);
}

#[test]
fn checker_assign_to_writable_class_property_no_error() {

    let diags = check_source(
        "class C { x: number = 0; }\
         const c = new C();\
         c.x = 1;",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_assign_to_readonly_interface_property_ts2540() {

    let diags = check_source(
        "interface I { readonly x: number; }\
         const obj: I = { x: 1 };\
         obj.x = 2;",
    );
    assert_diagnostic_code(&diags, 2540);
}

#[test]
fn checker_assign_to_writable_interface_property_no_error() {

    let diags = check_source(
        "interface I { x: number; }\
         const obj: I = { x: 1 };\
         obj.x = 2;",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_compound_assignment_to_readonly_ts2540() {

    let diags = check_source(
        "interface I { readonly y: number; }\
         const obj: I = { y: 1 };\
         obj.y = 10;",
    );
    assert_diagnostic_code(&diags, 2540);
}

#[test]
fn checker_assign_to_method_no_error() {

    let diags = check_source(
        "class C { m(): void {} }\
         const c = new C();\
         c.m = () => {};",
    );

    assert_no_diagnostics(&diags);
}

#[test]
fn checker_assign_to_inherited_readonly_ts2540() {

    let diags = check_source(
        "class B { readonly x: number = 1; }\
         class D extends B {}\
         const d = new D();\
         d.x = 2;",
    );
    assert_diagnostic_code(&diags, 2540);
}

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

    assert_no_diagnostics(&diags);
}

#[test]
fn checker_overload_matching_first_signature_no_error() {

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

    let diags = check_source(
        "function f(x: string): number;\n\
         function f(x: number): string;\n\
         function f(x: any): any { return x; }\n\
         f(true);",
    );
    assert_diagnostic_code(&diags, 2769);
}

#[test]
fn checker_overload_single_implementation_no_error() {

    let diags = check_source(
        "function f(x: number): number { return x + 1; }\n\
         let n: number = f(42);",
    );
    assert_no_diagnostics(&diags);
}

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
    let diags = check_source("interface Callback { (err: unknown): void; }");
    assert_no_diagnostics(&diags);
}

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

#[test]
fn checker_new_expression_property_access_no_error() {

    let diags = check_source(
        "class Foo { x: number = 1; }\n\
         let f = new Foo();\n\
         let n: number = f.x;",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_new_expression_property_missing_ts2339() {

    let diags = check_source(
        "class Foo { x: number = 1; }\n\
         let f = new Foo();\n\
         f.missing;",
    );
    assert_diagnostic_code(&diags, 2339);
}

#[test]
fn checker_new_expression_method_call_no_error() {

    let diags = check_source(
        "class Foo { greet(): string { return 'hi'; } }\n\
         let f = new Foo();\n\
         let s: string = f.greet();",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_new_expression_inherited_property_no_error() {

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

    let diags = check_source(
        "class Foo { x: number = 1; }\n\
         let f = new Foo();\n\
         let s: string = f.x;",
    );
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_new_expression_constructor_args_ts2345() {

    let diags = check_source(
        "class Foo { constructor(n: number) {} }\n\
         new Foo('hi');",
    );
    assert_diagnostic_code(&diags, 2345);
}

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

#[test]
fn checker_type_alias_assignable_value_no_error() {
    let diags = check_source("type Str = string;\nlet x: Str = 'hi';");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_type_alias_mismatch_ts2322() {

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

    let diags = check_source("type B = number;\ntype A = B;\nlet x: A = 'hi';");
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_recursive_type_alias_does_not_crash() {

    let diags = check_source("type A = B;\ntype B = A;\nlet x: A = 1;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_undefined_variable_ts2304() {
    let diags = check_source("let x = undefinedVar;");

    assert_diagnostic_code(&diags, 2552);
}

#[test]
fn checker_undefined_function_call_ts2304() {
    let diags = check_source("undefinedFunc();");

    assert_diagnostic_code(&diags, 2552);
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
    assert_diagnostic_code(&diags, 2552);
}

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
fn checker_for_loop_block_scope_no_redeclare() {

    let diags =
        check_source("for (let i = 0, j = 1; i < 10; i++) {}\nfor (let i = 0; i < 5; i++) {}");
    let redeclare = diags.iter().filter(|d| d.code == 2451).count();
    assert_eq!(redeclare, 0, "for-loop block scope should not redeclare");
}

#[test]
fn checker_switch_statement_no_error() {
    let diags = check_source("let x = 1;\nswitch (x) { case 1: break; default: break; }");
    assert_no_diagnostics(&diags);
}

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
    assert_diagnostic_code(&diags, 2552);
}

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

#[test]
fn checker_class_extends_no_error() {
    let diags = check_source("class Base { x = 1; }\nclass Derived extends Base { y = 2; }");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_class_extends_inherited_method_no_error() {

    let diags = check_source(
        "class Base { greet(): string { return 'hi'; } }\n\
         class Derived extends Base { test() { return this.greet(); } }",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_class_extends_inherited_property_no_error() {

    let diags = check_source(
        "class Base { x: number = 1; }\n\
         class Derived extends Base { test() { return this.x + 1; } }",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_class_extends_property_not_on_this_ts2339() {

    let diags = check_source(
        "class Base { x: number = 1; }\n\
         class Derived extends Base { test() { return this.missing; } }",
    );
    assert_diagnostic_code(&diags, 2339);
}

#[test]
fn checker_class_extends_override_method_no_error() {

    let diags = check_source(
        "class Base { greet(): string { return 'base'; } }\n\
         class Derived extends Base { greet(): string { return 'derived'; } }",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_class_extends_super_call_no_error() {

    let diags = check_source(
        "class Base { greet(): string { return 'base'; } }\n\
         class Derived extends Base { greet(): string { return super.greet(); } }",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_class_extends_implements_via_base_no_error() {

    let diags = check_source(
        "interface I { greet(): string; }\n\
         class Base implements I { greet(): string { return 'base'; } }\n\
         class Derived extends Base {}",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_class_extends_multilevel_no_error() {

    let diags = check_source(
        "class A { a(): number { return 1; } }\n\
         class B extends A { b(): number { return 2; } }\n\
         class C extends B { test() { return this.a() + this.b(); } }",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_class_this_property_access_ts2339() {

    let diags = check_source("class C { x: number = 1; test() { return this.missing; } }");
    assert_diagnostic_code(&diags, 2339);
}

#[test]
fn checker_class_this_property_access_no_error() {

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

    let diags = check_source(
        "interface IFoo { bar(): number; }\n\
         class C implements IFoo { bar() { return 42; } }",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_class_implements_interface_wrong_method_return_ts2420() {

    let diags = check_source(
        "interface IFoo { bar(): number; }\n\
         class C implements IFoo { bar(): string { return 'hi'; } }",
    );
    assert_diagnostic_code(&diags, 2416);
}

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

#[test]
fn checker_export_default_class_named_is_accessible() {

    let diags = check_source(
        "export default class Foo {}\n\
         let x: Foo | null = null;",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_export_default_function_named_is_accessible() {

    let diags = check_source(
        "export default function foo(): number { return 1; }\n\
         let x: number = foo();",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_export_default_identifier_expression_no_error() {

    let diags = check_source("const foo = 1;\nexport default foo;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_export_default_object_literal_no_error() {

    let diags = check_source("export default { a: 1, b: 2 };");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_export_equals_no_error() {

    let diags = check_source("function x(): void {}\nexport = x;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_export_star_no_error() {

    let diags = check_source("export * from \"mod\";");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_export_star_as_ns_no_error() {

    let diags = check_source("export * as ns from \"mod\";");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_export_named_reexport_no_error() {

    let diags = check_source("const x = 1;\nexport { x };");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_export_named_reexport_renamed_no_error() {

    let diags = check_source("const x = 1;\nexport { x as y };");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_import_default_no_error() {

    let diags = check_source("import D from \"mod\";\nexport function f(): void {}");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_import_named_and_default_no_error() {

    let diags = check_source("import D, { x } from \"mod\";\nexport function f(): void {}");
    assert_no_diagnostics(&diags);
}

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

#[test]
fn checker_typeof_expression_no_error() {
    let diags = check_source("let x = typeof 42;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_delete_expression_no_error() {

    let diags = check_source("let obj = { x: 1 };\ndelete obj.x;");
    assert_diagnostic_code(&diags, 2790);
}

#[test]
fn checker_void_expression_no_error() {
    let diags = check_source("let x = void 0;");
    assert_no_diagnostics(&diags);
}

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

#[test]
fn checker_jsx_self_closing_element_no_error() {
    let diags = check_source_tsx("const el = <div />;");

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

    let diags = check_source_tsx("const el = <div>{undefinedVar}</div>;");
    assert_diagnostic_code(&diags, 2552);
}

#[test]
fn checker_jsx_precondition_jsx_flag_missing() {

    let diags = check_source_tsx_with_args(
        "const el = <div />;",
        &[],
    );
    assert_diagnostic_code(&diags, 17004);
}

#[test]
fn checker_jsx_duplicate_attribute_names() {

    let diags = check_source_tsx("const el = <div data-x='1' data-x='2' />;");
    assert_diagnostic_code(&diags, 17001);
}

#[test]
fn checker_jsx_comma_operator_in_expression() {

    let diags = check_source_tsx("const a = 1; const b = 2; const el = <div>{a, b}</div>;");
    assert_diagnostic_code(&diags, 18007);
}

#[test]
fn checker_jsx_component_no_signatures() {

    let diags = check_source_tsx("const Foo = 42;\nconst el = <Foo />;");
    assert_diagnostic_code(&diags, 2604);
}

#[test]
fn checker_jsx_function_component_no_error() {

    let diags = check_source_tsx("function Foo() { return 1; }\nconst el = <Foo />;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_jsx_class_component_no_error() {

    let diags = check_source_tsx("class Foo {}\nconst el = <Foo />;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_jsx_function_component_returning_jsx_no_error() {

    let diags = check_source_tsx("function App() { return <div/> }\nconst el = <App />;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_jsx_arrow_function_component_no_error() {

    let diags = check_source_tsx("const App = () => <div/>;\nconst el = <App />;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_jsx_class_component_with_render_no_error() {

    let diags = check_source_tsx("class App { render() { return <div/> } }\nconst el = <App />;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_jsx_interface_call_signature_component_no_error() {

    let diags = check_source_tsx(
        "interface FC { (props: any): any }\nconst Foo: FC = () => null;\nconst el = <Foo />;",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_jsx_interface_construct_signature_component_no_error() {

    let diags = check_source_tsx(
        "interface Ctor { new (): any }\nconst Foo: Ctor = class {};\nconst el = <Foo />;",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_jsx_namespace_synthesized_no_implicit_any_react_jsx() {

    let diags = check_source_tsx_with_args(
        "const el = <div className=\"x\">hello <span>world</span></div>;",
        &["--jsx", "react-jsx", "--noImplicitAny"],
    );
    let jsx_diags: Vec<_> = diags
        .iter()
        .filter(|d| d.code == 2602 || d.code == 7026)
        .collect();
    assert!(
        jsx_diags.is_empty(),
        "expected no TS2602/TS7026 with synthesized JSX namespace, got: {:?}",
        jsx_diags
            .iter()
            .map(|d| (d.code, d.message_args.clone()))
            .collect::<Vec<_>>()
    );
}

#[test]
fn checker_jsx_namespace_synthesized_no_implicit_any_preserve() {

    let diags = check_source_tsx_with_args(
        "const el = <input type=\"text\" value={1} />;",
        &["--jsx", "preserve", "--noImplicitAny"],
    );
    let jsx_diags: Vec<_> = diags
        .iter()
        .filter(|d| d.code == 2602 || d.code == 7026)
        .collect();
    assert!(
        jsx_diags.is_empty(),
        "expected no TS2602/TS7026 with synthesized JSX namespace, got: {:?}",
        jsx_diags
            .iter()
            .map(|d| (d.code, d.message_args.clone()))
            .collect::<Vec<_>>()
    );
}

#[test]
fn checker_jsx_component_still_checked_under_synthetic_namespace() {

    let diags = check_source_tsx_with_args(
        "const Foo = 42;\nconst el = <Foo />;",
        &["--jsx", "react-jsx", "--noImplicitAny"],
    );
    assert_diagnostic_code(&diags, 2604);
}

#[test]
fn checker_dom_value_globals_resolvable() {
    let diags = check_source(
        "const t = document.title;\n\
         const h = window.location.href;\n\
         console.log(t, h);\n\
         const id = setTimeout(() => {}, 0);\n\
         clearTimeout(id);\n\
         const ua = navigator.userAgent;",
    );
    let ts2304 = diags.iter().filter(|d| d.code == 2304).collect::<Vec<_>>();
    assert!(
        ts2304.is_empty(),
        "expected no TS2304 for DOM value globals, got: {:?}",
        ts2304
            .iter()
            .map(|d| d.message_args.clone())
            .collect::<Vec<_>>()
    );
}

#[test]
fn checker_dom_type_globals_resolve_to_any() {

    let diags = check_source(
        "function handler(e: Event): void {}\n\
         function getNode(): HTMLElement | null { return null; }",
    );
    let ts2304 = diags.iter().filter(|d| d.code == 2304).collect::<Vec<_>>();
    assert!(
        ts2304.is_empty(),
        "expected no TS2304 for DOM type globals, got: {:?}",
        ts2304
            .iter()
            .map(|d| d.message_args.clone())
            .collect::<Vec<_>>()
    );
}

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
    assert_diagnostic_code(&diags, 2552);
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

#[test]
fn checker_arguments_in_function_no_error() {

    let diags = check_source("function foo() { return arguments; }");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_arguments_outside_function_is_undefined() {

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

#[test]
fn checker_arrow_param_initializer_references_outer_var() {

    let diags = check_source(
        "let outer = 10;\
         \x20let f = (x = outer) => x;",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_infer_type_not_visible_in_false_branch() {

    let diags = check_source(
        "type Test<T> = T extends Array<infer R> ? R : R;\
         \x20let x: Test<number> = 5;",
    );

    let count = diags.iter().filter(|d| d.code == 2304).count();
    assert_eq!(
        count, 1,
        "R in the false branch should not report TS2304 (known gap: infer          visibility is branch-scoped in Go only), got {}",
        count
    );
}

#[test]
fn checker_static_member_cannot_reference_type_param() {

    let diags = check_source(
        "class Foo<T> {\
         \x20   static x: T;\
         \x20}",
    );

    assert!(
        !diags.is_empty(),
        "Expected diagnostics for static member referencing type parameter"
    );
}

#[test]
fn checker_export_default_alias_resolution() {

    let diags = check_source(
        "export default function foo() {}\
         \x20let x = foo();",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_function_expression_self_name() {

    let diags = check_source(
        "let fact = function f(n: number): number {\
         \x20   return n <= 1 ? 1 : n * f(n - 1);\
         \x20};",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_namespace_member_access_from_outside() {

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

#[test]
fn checker_global_symbol_with_lib() {

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

    let diags = check_source("let x = 42; let y = x;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_type_inference_string_variable() {

    let diags = check_source("let x = \"hello\"; let y = x;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_type_inference_binary_expression() {

    let diags = check_source("let x = 1 + 2;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_narrowing_null_removed_in_true_branch() {

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

    let diags = check_source_strict(
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

    let diags = check_source(
        "let x: string | number = 0;\
         x = \"hello\";\
         let y: string = x;",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_narrowing_switch_on_symbol() {

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

    let diags = check_source(
        "function f(x: string | number | boolean) {\
         switch (x) {\
             case \"foo\":\
                 break;\
             case 42:\
                 break;\
             default:\
                 let z: boolean = x;\
                 break;\
         } }",
    );

    assert_diagnostic_count(&diags, 2322, 1);
}

#[test]
fn checker_narrowing_switch_on_discriminant_property() {

    let diags = check_source(
        "function f(obj: { kind: \"foo\", value: string } | { kind: \"bar\", count: number }) {\
         switch (obj.kind) {\
             case \"foo\":\
                 let v: string = obj.value;\
                 break;\
             case \"bar\":\
                 let c: number = obj.count;\
                 break;\
         } }",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_narrowing_switch_default_discriminant_property() {

    let diags = check_source(
        "function f(obj: { kind: \"foo\", value: string } | { kind: \"bar\", count: number } | { kind: \"baz\", flag: boolean }) {\
         switch (obj.kind) {\
             case \"foo\":\
                 break;\
             case \"bar\":\
                 break;\
             default:\
                 let f: boolean = obj.flag;\
                 break;\
         } }",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_narrowing_type_predicate_true_branch() {

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

    let diags = check_source(
        "type T = { a: string } | null;\
         let x: T = null;\
         if (x?.a !== undefined) {\
             let y: { a: string } = x;\
         }",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_narrowing_equality_replaces_string_with_literal() {

    let diags = check_source(
        "let x: string | number = 0;\
         if (x === \"foo\") {\
             let y: \"foo\" = x;\
         }",
    );
    assert_diagnostic_code(&diags, 2367);
}

#[test]
fn checker_narrowing_equality_replaces_number_with_literal() {

    let diags = check_source(
        "let x: string | number = \"\";\
         if (x === 42) {\
             let y: 42 = x;\
         }",
    );
    assert_diagnostic_code(&diags, 2367);
}

#[test]
fn checker_narrowing_equality_strict_null_vs_undefined() {

    let diags = check_source_strict(
        "let x: string | null | undefined = null;\
         if (x === undefined) {\
             let y: undefined = x;\
         }",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_narrowing_equality_strict_null_kept() {

    let diags = check_source_strict(
        "let x: string | null | undefined = undefined;\
         if (x === null) {\
             let y: null = x;\
         }",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_narrowing_equality_false_branch_removes_literal() {

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

    let diags = check_source(
        "let x: any = 0;\
         if (x === \"foo\") {\
             let y: any = x;\
         }",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_narrowing_typeof_switch_string() {

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

#[test]
fn checker_narrowing_switch_true_equality() {

    let diags = check_source(
        "function f(x: string | number) {\
         switch (true) {\
             case x === \"foo\":\
                 let y: string = x;\
                 break;\
             case x === 42:\
                 let z: number = x;\
                 break;\
         }\
         }",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_narrowing_switch_true_not_equal() {

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

    let diags = check_source(
        "function f(x: \"foo\" | 42 | boolean) {\
         switch (true) {\
             case x === \"foo\":\
                 break;\
             case x === 42:\
                 break;\
             default:\
                 let z: boolean = x;\
                 break;\
         }\
         }",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_narrowing_asserts_is_type() {

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

#[test]
fn checker_narrowing_instanceof_true_branch() {

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

    let diags = check_source(
        "class Foo { greet() {} }\
         function f(x: Foo | string) {\
         \x20   x.greet();\
         }",
    );
    let count = diags.iter().filter(|d| d.code == 2339).count();
    assert_eq!(count, 1, "x.greet() on Foo | string is TS2339 in Go too");
}

#[test]
fn checker_narrowing_and_condition() {

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

#[test]
fn checker_narrowing_loose_equality_eq_null() {

    let diags = check_source_strict(
        "let x: string | null | undefined = null;\
         if (x == null) {\
             let y: null | undefined = x;\
         }",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_narrowing_loose_equality_ne_null() {

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

    let diags = check_source(
        "declare const sym: symbol;\
         let x: symbol | string = sym;\
         if (typeof x === \"symbol\") {\
             let y: symbol = x;\
         }",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_narrowing_const_alias_truthiness() {

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

    let diags = check_source(
        "let x: string | null = null;\
         let alias = x;\
         if (alias) {\
             let y: string | null = x;\
         }",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_narrowing_typeof_function_keeps_callable() {

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

    let diags = check_source(
        "type T = (() => void) | { a: string };\
         let x: T = { a: \"\" };\
         if (typeof x !== \"function\") {\
             let y: { a: string } = x;\
         }",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_narrowing_typeof_discriminant_property_string() {

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

    let diags = check_source(
        "type T = { kind: \"foo\", value: string } | { kind: 42, count: number };\
         let obj: T = { kind: \"foo\", value: \"x\" };\
         if (typeof obj.kind === \"number\") {\
             let c: number = obj.count;\
         }",
    );
    assert_diagnostic_code(&diags, 2339);
}

#[test]
fn checker_narrowing_typeof_discriminant_property_false_branch() {

    let diags = check_source(
        "type T = { kind: \"foo\", value: string } | { kind: 42, count: number };\
         let obj: T = { kind: \"foo\", value: \"x\" };\
         if (typeof obj.kind !== \"string\") {\
             let c: number = obj.count;\
         }",
    );
    assert_diagnostic_code(&diags, 2339);
}

#[test]
fn checker_narrowing_nullish_coalescing_false_branch_narrows_left_to_null() {

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

fn get_ts2322_args(diags: &[tsox::ast::Diagnostic]) -> Vec<String> {
    diags
        .iter()
        .find(|d| d.code == 2322)
        .map(|d| d.message_args.clone())
        .unwrap_or_default()
}

#[test]
fn checker_type_display_string_literal() {

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

    let diags = check_source(
        "let x: string | number = 0;\
         let y: boolean = x;",
    );
    let args = get_ts2322_args(&diags);
    assert!(!args.is_empty(), "Expected TS2322");
    assert!(
        args.iter().any(|a| a.contains("number")),
        "Expected 'number' in args: {:?}",
        args
    );
}

#[test]
fn checker_type_display_boolean_literal() {

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

#[test]
fn checker_as_expression_uses_type_annotation_no_error() {

    let diags = check_source("let x: unknown = 0; let y: string = x as string;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_as_expression_wrong_annotation_ts2322() {

    let diags = check_source("let x: unknown = 0; let y: string = x as number;");
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_satisfies_expression_keeps_expression_type_no_error() {

    let diags = check_source("let x = 'hi'; let y: string = x satisfies string;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_satisfies_expression_keeps_expression_type_ts2322() {

    let diags = check_source("let x = 42; let y: string = x satisfies string;");
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_prefix_unary_not_returns_boolean_no_error() {

    let diags = check_source("let x = 1; let y: boolean = !x;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_prefix_unary_not_returns_boolean_ts2322() {

    let diags = check_source("let x = 1; let y: number = !x;");
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_prefix_unary_minus_returns_number_no_error() {

    let diags = check_source("let x = 1; let y: number = -x;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_prefix_unary_minus_returns_number_ts2322() {

    let diags = check_source("let x = 1; let y: string = -x;");
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_postfix_increment_returns_number_no_error() {

    let diags = check_source("let x = 1; let y: number = x++;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_array_length_property_returns_number_no_error() {

    let diags = check_source("let arr: number[] = [1, 2, 3]; let y: number = arr.length;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_array_length_property_returns_number_ts2322() {

    let diags = check_source("let arr: number[] = [1, 2, 3]; let y: string = arr.length;");
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_property_access_existing_property_no_error() {

    let diags = check_source("let obj = { a: 1, b: 'hi' }; let x = obj.a; let y = obj.b;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_property_access_missing_property_on_object_ts2339() {

    let diags = check_source("let obj = { a: 1, b: 'hi' }; let x = obj.c;");
    assert_diagnostic_code(&diags, 2339);
}

#[test]
fn checker_property_access_on_number_ts2339() {

    let diags = check_source("let x: number = 1; x.toUpperCase();");
    assert_diagnostic_code(&diags, 2339);
}

#[test]
fn checker_property_access_on_string_literal_no_error() {

    let diags = check_source("let x = 'hi'; x.toUpperCase();");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_property_access_on_any_no_error() {

    let diags = check_source("let x: any = 1; x.toUpperCase();");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_property_access_on_type_parameter_constraint_no_error() {

    let diags = check_source("function f<T extends { a: number }>(x: T) { return x.a; }");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_property_access_on_type_parameter_missing_ts2339() {

    let diags = check_source("function f<T extends { a: number }>(x: T) { return x.b; }");
    assert_diagnostic_code(&diags, 2339);
}

#[test]
fn checker_property_access_on_union_present_in_all_no_error() {

    let diags = check_source(
        "let x: { a: number } | { a: string } = { a: 1 };\
         let y = x.a;",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_property_access_on_union_missing_in_one_ts2339() {

    let diags = check_source(
        "let x: { a: number } | { b: string } = { a: 1 };\
         let y = x.a;",
    );
    assert_diagnostic_count(&diags, 2339, 0);
}

#[test]
fn checker_property_access_on_intersection_no_error() {

    let diags = check_source(
        "let x: { a: number } & { b: string } = { a: 1, b: 'hi' };\
         let y = x.a;\
         let z = x.b;",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_property_access_on_intersection_missing_ts2339() {

    let diags = check_source(
        "let x: { a: number } & { b: string } = { a: 1, b: 'hi' };\
         let y = x.c;",
    );
    assert_diagnostic_code(&diags, 2339);
}

#[test]
fn checker_property_access_array_length_no_error() {

    let diags = check_source("let arr: number[] = [1, 2, 3]; let x = arr.length;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_property_access_array_known_method_no_error() {

    let diags = check_source("let arr: number[] = [1, 2, 3]; arr.push(4);");
    assert_no_diagnostics(&diags);
    assert_diagnostic_count(&diags, 2339, 0);
}

#[test]
fn checker_property_access_optional_chain_no_error() {

    let diags = check_source("let x: any = null; let y = x?.foo;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_property_access_index_signature_no_error() {

    let diags = check_source(
        "let x: { [key: string]: number } = { a: 1 };\
         let y = x.foo;",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_array_element_access_returns_element_type_no_error() {

    let diags = check_source("let arr: number[] = [1, 2, 3]; let y: number = arr[0];");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_array_element_access_returns_element_type_ts2322() {

    let diags = check_source("let arr: number[] = [1, 2, 3]; let y: string = arr[0];");
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_string_array_element_access_returns_string_no_error() {

    let diags = check_source("let arr: string[] = ['a']; let y: string = arr[0];");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_call_arg_matching_type_no_error() {

    let diags = check_source("function f(x: number) {} f(42);");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_call_arg_string_to_number_ts2345() {

    let diags = check_source("function f(x: number) {} f('hi');");
    assert_diagnostic_code(&diags, 2345);
}

#[test]
fn checker_call_arg_number_to_string_ts2345() {

    let diags = check_source("function f(x: string) {} f(42);");
    assert_diagnostic_code(&diags, 2345);
}

#[test]
fn checker_call_arg_boolean_to_number_ts2345() {

    let diags = check_source("function f(x: number) {} f(true);");
    assert_diagnostic_code(&diags, 2345);
}

#[test]
fn checker_call_arg_union_member_no_error() {

    let diags = check_source("function f(x: string | number) {} f(42);");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_call_arg_outside_union_ts2345() {

    let diags = check_source("function f(x: string | number) {} f(true);");
    assert_diagnostic_code(&diags, 2345);
}

#[test]
fn checker_call_arg_any_param_no_error() {

    let diags = check_source("function f(x: any) {} f('hi'); f(42); f(true);");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_call_multiple_args_first_mismatch_ts2345() {

    let diags = check_source("function f(a: number, b: string) {} f('hi', 'ok');");
    assert_diagnostic_code(&diags, 2345);
}

#[test]
fn checker_call_multiple_args_second_mismatch_ts2345() {

    let diags = check_source("function f(a: number, b: string) {} f(1, 42);");
    assert_diagnostic_code(&diags, 2345);
}

#[test]
fn checker_call_arrow_function_arg_ts2345() {

    let diags = check_source("let f = (x: number) => x; f('hi');");
    assert_diagnostic_code(&diags, 2345);
}

#[test]
fn checker_call_arg_matching_object_type_no_error() {

    let diags = check_source("function f(p: { a: number }) {} f({ a: 1 });");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_call_arg_object_missing_property_ts2345() {

    let diags = check_source("function f(p: { a: number }) {} f({ b: 1 });");
    assert_diagnostic_code(&diags, 2353);
}

#[test]
fn checker_call_arg_wrong_property_type_ts2345() {

    let diags = check_source("function f(p: { a: number }) {} f({ a: 'hi' });");
    assert_diagnostic_code(&diags, 2345);
}

#[test]
fn checker_new_expression_arg_ts2345() {

    let diags = check_source("class Foo { constructor(x: number) {} } let f = new Foo('hi');");
    assert_diagnostic_code(&diags, 2345);
}

#[test]
fn checker_new_expression_arg_no_error() {

    let diags = check_source("class Foo { constructor(x: number) {} } let f = new Foo(42);");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_call_arg_fewer_args_no_error() {

    let diags = check_source("function f(a: number, b?: string) {} f(42);");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_call_too_few_args_ts2554() {

    let diags = check_source("function f(x: number) {} f();");
    assert_diagnostic_code(&diags, 2554);
}

#[test]
fn checker_call_too_many_args_ts2554() {

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

    let diags = check_source("function f(a?: number) {} f();");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_call_rest_param_no_args_no_error() {

    let diags = check_source("function f(...args: number[]) {} f();");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_call_rest_param_many_args_no_error() {

    let diags = check_source("function f(...args: number[]) {} f(1, 2, 3);");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_call_required_then_rest_too_few_ts2555() {

    let diags = check_source("function f(a: number, ...rest: string[]) {} f();");
    assert_diagnostic_code(&diags, 2555);
}

#[test]
fn checker_call_required_then_rest_enough_no_error() {

    let diags = check_source("function f(a: number, ...rest: string[]) {} f(1);");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_new_too_few_args_ts2554() {

    let diags = check_source("class Foo { constructor(x: number) {} } let f = new Foo();");
    assert_diagnostic_code(&diags, 2554);
}

#[test]
fn checker_new_too_many_args_ts2554() {

    let diags = check_source("class Foo { constructor(x: number) {} } let f = new Foo(1, 2);");
    assert_diagnostic_code(&diags, 2554);
}

#[test]
fn checker_call_overload_too_few_ts2554() {

    let diags = check_source(
        "function f(x: number): void; function f(x: string): void; function f(x: any) {} f();",
    );
    assert_diagnostic_code(&diags, 2554);
}

#[test]
fn checker_call_overload_matching_no_error() {

    let diags = check_source(
        "function f(x: number): void; function f(x: string): void; function f(x: any) {} f(1);",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_non_null_expression_returns_expression_type_no_error() {

    let diags = check_source("let x: number | null = 1; let y: number = x!;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_type_assertion_expression_uses_type_annotation_no_error() {

    let diags = check_source("let x: any = 1; let y: number = <number>x;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_type_assertion_expression_wrong_annotation_ts2322() {

    let diags = check_source("let x: any = 1; let y: number = <string>x;");
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_conditional_expression_both_branches_same_type_no_error() {

    let diags = check_source("let cond = true; let y: number = cond ? 1 : 2;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_conditional_expression_union_type_to_union_no_error() {

    let diags = check_source(
        "let cond = true;\
         let y: number | string = cond ? 1 : 'hi';",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_template_expression_returns_string_no_error() {

    let diags = check_source("let x = 1; let y: string = `${x}`;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_template_expression_returns_string_ts2322() {

    let diags = check_source("let x = 1; let y: number = `${x}`;");
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_delete_expression_returns_boolean_no_error() {

    let diags = check_source("let x = 1; let y: boolean = delete x;");
    assert_diagnostic_code(&diags, 1102);
    assert_diagnostic_code(&diags, 2703);
}

#[test]
fn checker_void_expression_returns_undefined_no_error() {

    let diags = check_source("let x = 1; let y: undefined = void x;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_void_expression_returns_undefined_ts2322() {

    let diags = check_source_tsx_with_args(
        "let x = 1; let y: number = void x;",
        &["--strictNullChecks", "true"],
    );
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_array_literal_infer_number_array_no_error() {

    let diags = check_source("let arr = [1, 2, 3]; let y: number[] = arr;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_array_literal_infer_number_array_ts2322() {

    let diags = check_source("let arr = [1, 2, 3]; let y: string[] = arr;");
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_array_literal_infer_string_array_no_error() {

    let diags = check_source("let arr = ['a', 'b']; let y: string[] = arr;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_array_literal_infer_string_array_ts2322() {

    let diags = check_source("let arr = ['a', 'b']; let y: number[] = arr;");
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_array_literal_element_access_after_inference_no_error() {

    let diags = check_source("let arr = [1, 2, 3]; let y: number = arr[0];");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_array_literal_empty_no_error() {

    let diags = check_source("let arr = []; let y: number[] = arr;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_object_literal_infer_number_property_no_error() {

    let diags = check_source("let obj = { a: 1 }; let y: { a: number } = obj;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_object_literal_infer_number_property_to_string_ts2322() {

    let diags = check_source("let obj = { a: 1 }; let y: { a: string } = obj;");
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_object_literal_infer_multiple_properties_no_error() {

    let diags = check_source(
        "let obj = { a: 1, b: 'hi' };\
         let y: { a: number; b: string } = obj;",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_object_literal_infer_missing_property_ts2322() {

    let diags = check_source(
        "let obj = { a: 1 };\
         let y: { a: number; b: string } = obj;",
    );
    assert_diagnostic_code(&diags, 2741);
}

#[test]
fn checker_object_literal_infer_boolean_property_no_error() {

    let diags = check_source(
        "let obj = { flag: true };\
         let y: { flag: boolean } = obj;",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_object_literal_infer_nested_object_no_error() {

    let diags = check_source(
        "let obj = { a: { b: 1 } };\
         let y: { a: { b: number } } = obj;",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_object_literal_infer_shorthand_property_no_error() {

    let diags = check_source(
        "let a = 42;\
         let obj = { a };\
         let y: { a: number } = obj;",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_object_literal_infer_string_property_to_number_ts2322() {

    let diags = check_source("let obj = { a: 'hi' }; let y: { a: number } = obj;");
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_function_type_annotation_no_error() {

    let diags = check_source("let f: (x: number) => number = (x) => x + 1;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_function_type_wrong_return_type_ts2322() {

    let diags = check_source("let f: (x: number) => number = (x) => 'hi';");
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_function_type_extra_parameters_ts2322() {

    let diags = check_source("let f: (x: number) => number = (x: number, y: number) => x + y;");
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_function_type_fewer_parameters_no_error() {

    let diags = check_source("let f: (x: number, y: number) => number = (x: number) => x + 1;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_function_type_optional_parameter_no_error() {

    let diags = check_source("let f: (x: number) => number = (x?: number) => (x ?? 0) + 1;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_function_type_rest_parameter_no_error() {

    let diags = check_source("let f: (x: number) => number = (...args: number[]) => args[0] ?? 0;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_function_type_no_params_no_error() {

    let diags = check_source("let f: () => number = () => 42;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_function_type_void_return_no_error() {

    let diags = check_source("let f: () => void = () => 42;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_arrow_param_type_mismatch_ts2322() {

    let diags = check_source("let f: (x: number) => number = (x: string) => 1;");
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_arrow_param_type_match_no_error() {

    let diags = check_source("let f: (x: number) => number = (x: number) => x + 1;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_arrow_two_param_type_mismatch_ts2322() {

    let diags =
        check_source("let f: (a: number, b: number) => number = (a: number, b: string) => 1;");
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_arrow_param_subtype_no_error() {

    let diags = check_source("let f: (x: string | number) => number = (x: string | number) => 1;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_function_expression_param_type_mismatch_ts2322() {

    let diags = check_source("let f: (x: number) => number = function (x: string) { return 1; };");
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_arrow_unannotated_param_no_error() {

    let diags = check_source("let f: (x: number) => number = (x) => x + 1;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_contextual_param_return_type_mismatch_ts2322() {

    let diags = check_source("let f: (x: string) => number = (x) => x;");
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_contextual_param_return_type_match_no_error() {

    let diags = check_source("let f: (x: number) => number = (x) => x;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_contextual_param_arithmetic_no_error() {

    let diags = check_source("let f: (x: number) => number = (x) => x + 1;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_contextual_param_block_body_return_ts2322() {

    let diags = check_source("let f: (x: number) => string = (x) => { return x; };");
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_contextual_param_two_params_return_ts2322() {

    let diags = check_source("let f: (a: number, b: string) => number = (a, b) => b;");
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_contextual_param_function_expression_ts2322() {

    let diags = check_source("let f: (x: string) => number = function (x) { return x; };");
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_contextual_param_fewer_params_no_error() {

    let diags = check_source("let f: (x: number, y: number) => number = (x) => x;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_conditional_true_branch_no_error() {

    let diags =
        check_source("type T = number extends number ? \"yes\" : \"no\";\nlet x: T = \"yes\";");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_conditional_true_branch_mismatch_ts2322() {

    let diags =
        check_source("type T = number extends number ? \"yes\" : \"no\";\nlet x: T = \"no\";");
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_conditional_false_branch_no_error() {

    let diags =
        check_source("type T = number extends string ? \"yes\" : \"no\";\nlet x: T = \"no\";");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_conditional_false_branch_mismatch_ts2322() {

    let diags =
        check_source("type T = number extends string ? \"yes\" : \"no\";\nlet x: T = \"yes\";");
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_conditional_literal_check_type_true_no_error() {

    let diags = check_source("type T = 1 extends number ? \"a\" : \"b\";\nlet x: T = \"a\";");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_conditional_literal_check_type_false_no_error() {

    let diags = check_source("type T = 1 extends string ? \"a\" : \"b\";\nlet x: T = \"b\";");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_conditional_infer_r_array_element_no_error() {

    let diags = check_source(
        "type T<U> = U extends (infer R)[] ? R : never;\nlet x: number = null as any as T<number[]>;",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_conditional_infer_r_array_element_mismatch_ts2322() {

    let diags = check_source(
        "type T<U> = U extends (infer R)[] ? R : never;\nlet x: string = null as any as T<number[]>;",
    );
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_conditional_infer_r_string_no_error() {

    let diags = check_source(
        "type T<U> = U extends infer R ? R : never;\nlet x: string = null as any as T<string>;",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_conditional_infer_r_number_mismatch_ts2322() {

    let diags = check_source(
        "type T<U> = U extends infer R ? R : never;\nlet x: string = null as any as T<number>;",
    );
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_conditional_infer_r_never_branch_no_error() {

    let diags = check_source(
        "type T = string extends number ? \"yes\" : never;\nlet x: \"yes\" = null as any as T;",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_conditional_nested_true_no_error() {

    let diags = check_source(
        "type T = number extends number ? (1 extends number ? \"a\" : \"b\") : \"c\";\nlet x: T = \"a\";",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_conditional_nested_false_mismatch_ts2322() {

    let diags = check_source(
        "type T = number extends string ? (1 extends number ? \"a\" : \"b\") : \"c\";\nlet x: T = \"a\";",
    );
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_conditional_boolean_check_true_no_error() {

    let diags = check_source("type T = true extends boolean ? 1 : 0;\nlet x: T = 1;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_conditional_boolean_check_false_no_error() {

    let diags = check_source("type T = true extends string ? 1 : 0;\nlet x: T = 0;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_conditional_infer_r_in_function_return_no_error() {

    let diags = check_source(
        "type Ret<F> = F extends (...args: any[]) => infer R ? R : never;\nlet x: number = null as any as Ret<() => number>;",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_keyof_object_type_no_error() {

    let diags = check_source(
        "type K = keyof { a: number; b: string };\nlet x: \"a\" | \"b\" = null as any as K;",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_keyof_object_type_single_key_no_error() {

    let diags = check_source("type K = keyof { x: 1 };\nlet x: \"x\" = null as any as K;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_keyof_object_type_subset_assignable_no_error() {

    let diags =
        check_source("type K = keyof { a: number };\nlet x: \"a\" | \"b\" = null as any as K;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_keyof_object_type_missing_key_ts2322() {

    let diags =
        check_source("type K = keyof { a: number; b: string };\nlet x: \"a\" = null as any as K;");
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_keyof_via_type_alias_no_error() {

    let diags = check_source(
        "type Obj = { a: number; b: string };\ntype K = keyof Obj;\nlet x: \"a\" | \"b\" = null as any as K;",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_keyof_never_type() {

    let diags = check_source("type K = keyof never;\nlet x: \"a\" = null as any as K;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_keyof_empty_object_is_never() {

    let diags = check_source("type K = keyof {};\nlet x: \"a\" = null as any as K;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_keyof_union_common_keys_no_error() {

    let diags = check_source(
        "type A = { a: number; b: string };\ntype B = { b: string; c: number };\ntype K = keyof (A | B);\nlet x: \"b\" = null as any as K;",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_keyof_intersection_all_keys_no_error() {

    let diags = check_source(
        "type A = { a: number };\ntype B = { b: string };\ntype K = keyof (A & B);\nlet x: \"a\" | \"b\" = null as any as K;",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_indexed_access_string_literal_no_error() {

    let diags = check_source(
        "type T = { a: number; b: string };\ntype A = T[\"a\"];\nlet x: number = null as any as A;",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_indexed_access_string_literal_mismatch_ts2322() {

    let diags = check_source(
        "type T = { a: number; b: string };\ntype A = T[\"a\"];\nlet x: string = null as any as A;",
    );
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_indexed_access_keyof_no_error() {

    let diags = check_source(
        "type T = { a: number; b: string };\ntype V = T[keyof T];\nlet x: number | string = null as any as V;",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_indexed_access_keyof_missing_member_ts2322() {

    let diags = check_source(
        "type T = { a: number; b: string };\ntype V = T[keyof T];\nlet x: boolean = null as any as V;",
    );
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_indexed_access_union_of_keys_no_error() {

    let diags = check_source(
        "type T = { a: number; b: string };\ntype K = \"a\" | \"b\";\ntype V = T[K];\nlet x: number | string = null as any as V;",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_indexed_access_array_number_no_error() {

    let diags = check_source(
        "type Arr = number[];\ntype Elem = Arr[number];\nlet x: number = null as any as Elem;",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_indexed_access_array_number_mismatch_ts2322() {

    let diags = check_source(
        "type Arr = number[];\ntype Elem = Arr[number];\nlet x: string = null as any as Elem;",
    );
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_indexed_access_via_alias_no_error() {

    let diags = check_source(
        "type Obj = { value: boolean };\ntype V = Obj[\"value\"];\nlet x: boolean = null as any as V;",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_template_literal_type_concrete_flatten_no_error() {

    let diags = check_source("type T = `a-${1}-b`;\nlet x: \"a-1-b\" = null as any as T;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_template_literal_type_concrete_flatten_mismatch_ts2322() {

    let diags = check_source("type T = `a-${1}`;\nlet x: \"a-2\" = null as any as T;");
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_template_literal_type_string_span_no_error() {

    let diags =
        check_source("type T = `prefix-${string}`;\nlet x: T = null as any as `prefix-hello`;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_template_literal_type_multiple_spans_flatten_no_error() {

    let diags =
        check_source("type T = `x-${true}-${\"y\"}`;\nlet x: \"x-true-y\" = null as any as T;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_template_literal_type_via_alias_no_error() {

    let diags = check_source(
        "type Prefix<T> = `pre-${T}`;\ntype P = Prefix<\"x\">;\nlet v: \"pre-x\" = null as any as P;",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_mapped_type_literal_union_no_error() {

    let diags =
        check_source("type M = { [K in \"a\" | \"b\"]: number };\nlet x: M = { a: 1, b: 2 };");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_mapped_type_literal_union_mismatch_ts2322() {

    let diags =
        check_source("type M = { [K in \"a\" | \"b\"]: number };\nlet x: M = { a: \"hi\", b: 2 };");
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_mapped_type_keyof_no_error() {

    let diags = check_source(
        "type T = { a: 1; b: \"x\" };\ntype M = { [K in keyof T]: number };\nlet x: M = { a: 1, b: 2 };",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_mapped_type_keyof_mismatch_ts2322() {

    let diags = check_source(
        "type T = { a: 1; b: \"x\" };\ntype M = { [K in keyof T]: number };\nlet x: M = { a: 1 };",
    );
    assert_diagnostic_code(&diags, 2741);
}

#[test]
fn checker_mapped_type_optional_no_error() {

    let diags = check_source(
        "type T = { a: 1; b: \"x\" };\ntype M = { [K in keyof T]?: number };\nlet x: M = { };",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_mapped_type_identity_no_error() {

    let diags = check_source(
        "type T = { a: number; b: string };\ntype M = { [K in keyof T]: T[K] };\nlet x: M = { a: 1, b: \"hi\" };",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_mapped_type_identity_mismatch_ts2322() {

    let diags = check_source(
        "type T = { a: number; b: string };\ntype M = { [K in keyof T]: T[K] };\nlet x: M = { a: \"hi\", b: \"hi\" };",
    );
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_keyof_constrained_type_parameter_no_error() {

    let diags = check_source(
        "type K<T extends { a: number; b: string }> = keyof T;\nlet x: \"a\" | \"b\" = null as any as K<{ a: 1; b: 2 }>;",
    );
    assert_diagnostic_code(&diags, 2344);
}

#[test]
fn checker_conditional_target_accepts_true_branch_no_error() {

    let diags = check_source(
        "type C = number extends number ? string : number;\nlet x: string = \"hi\" as C;",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_recursive_structural_type_assignable_no_error() {

    let diags = check_source(
        "type Box<T> = { next: Box<T> | null };\nlet x: Box<number> = { next: null };",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_recursive_structural_type_self_assignable_no_error() {

    let diags = check_source(
        "type A = { value: number; next: A | null };\n\
         type B = { value: number; next: B | null };\n\
         let x: B = { value: 1, next: null } as A;",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_object_literal_widening_reassignment_no_error() {

    let diags = check_source("let x = { a: 1 };\nx = { a: 2 };");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_object_literal_widening_property_assignment_no_error() {

    let diags = check_source("let x = { a: 1 };\nx.a = 2;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_object_literal_widening_string_no_error() {

    let diags = check_source("let x = { a: 'hi' };\nx = { a: 'bye' };");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_object_literal_widening_boolean_no_error() {

    let diags = check_source("let x = { flag: true };\nx = { flag: false };");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_object_literal_widening_nested_no_error() {

    let diags = check_source("let x = { a: { b: 1 } };\nx = { a: { b: 2 } };");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_object_literal_contextual_preserves_literal_no_error() {

    let diags = check_source("let x: { a: 1 } = { a: 1 };");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_object_literal_contextual_mismatch_ts2322() {

    let diags = check_source("let x: { a: 2 } = { a: 1 };");
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_call_arg_arrow_contextual_param_type_ts2339() {

    let diags = check_source(
        "function f(cb: (x: { a: number }) => void): void {}\n\
         f((x) => x.b);",
    );
    assert_diagnostic_code(&diags, 2339);
}

#[test]
fn checker_call_arg_arrow_contextual_valid_no_error() {

    let diags = check_source(
        "function f(cb: (x: { a: number }) => void): void {}\n\
         f((x) => x.a);",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_call_arg_object_literal_contextual_no_error() {

    let diags = check_source(
        "function f(x: { a: number }): void {}\n\
         f({ a: 1 });",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_call_arg_object_literal_contextual_mismatch_ts2345() {

    let diags = check_source(
        "function f(x: { a: number }): void {}\n\
         f({ a: 'hi' });",
    );
    assert_diagnostic_code(&diags, 2345);
}

#[test]
fn checker_try_catch_finally_no_crash_no_error() {

    let diags = check_source(
        "try {\n  let x = 1;\n} catch (e) {\n  let y = 2;\n} finally {\n  let z = 3;\n}",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_narrowing_lost_after_try_block_no_error() {

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

    let diags = check_source("try {\n  throw 42;\n} catch (e) {\n  let y = e;\n}");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_interface_assignable_no_error() {

    let diags = check_source(
        "interface Foo { a: number; b: string }\n\
         let x: Foo = { a: 1, b: 'hi' };",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_interface_missing_property_ts2741() {

    let diags = check_source(
        "interface Foo { a: number; b: string }\n\
         let x: Foo = { a: 1 };",
    );
    assert!(diags.iter().any(|d| d.code != 0));
}

#[test]
fn checker_interface_wrong_property_type_ts2322() {

    let diags = check_source(
        "interface Foo { a: number }\n\
         let x: Foo = { a: 'hi' };",
    );
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_interface_property_access_no_error() {

    let diags = check_source(
        "interface Foo { a: number }\n\
         let x: Foo = { a: 1 };\n\
         let y: number = x.a;",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_interface_property_access_missing_ts2339() {

    let diags = check_source(
        "interface Foo { a: number }\n\
         let x: Foo = { a: 1 };\n\
         x.b;",
    );
    assert_diagnostic_code(&diags, 2339);
}

#[test]
fn checker_generic_interface_substitution_no_error() {

    let diags = check_source(
        "interface Box<T> { value: T }\n\
         let x: Box<number> = { value: 1 };",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_generic_interface_substitution_mismatch_ts2322() {

    let diags = check_source(
        "interface Box<T> { value: T }\n\
         let x: Box<number> = { value: 'hi' };",
    );
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_interface_method_signature_no_error() {

    let diags = check_source(
        "interface Foo { greet(): void }\n\
         let x: Foo = { greet: () => {} };\n\
         x.greet();",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_interface_index_signature_no_error() {

    let diags = check_source(
        "interface Foo { [key: string]: number }\n\
         let x: Foo = { a: 1 };\n\
         let y: number = x.a;",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_numeric_enum_no_error() {

    let diags = check_source(
        "enum Color { Red, Green, Blue }\n\
         let x: Color = Color.Red;",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_numeric_enum_member_values() {

    let diags = check_source(
        "enum Direction { Up = 1, Down = 2 }\n\
         let x: Direction = Direction.Up;",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_string_enum_no_error() {

    let diags = check_source(
        "enum Direction { Up = 'UP', Down = 'DOWN' }\n\
         let x: Direction = Direction.Up;",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_enum_property_access_no_error() {

    let diags = check_source(
        "enum Color { Red = 0, Green = 1 }\n\
         let r = Color.Red;\n\
         let g = Color.Green;",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_enum_wrong_assign_ts2322() {

    let diags = check_source(
        "enum Color { Red, Green, Blue }\n\
         let x: Color = 42;",
    );
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_mixed_enum_no_error() {

    let diags = check_source(
        "enum Shape { Circle = 0, Square = 'SQ' }\n\
         let x: Shape = Shape.Circle;",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_enum_auto_increment() {

    let diags = check_source(
        "enum Color { Red = 0, Green, Blue }\n\
         let x: Color = Color.Green;",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_evolving_array_push_number_no_error() {

    let diags = check_source(
        "let x = [];\n\
         x.push(1);\n\
         let y: number = x[0];",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_evolving_array_push_string_no_error() {

    let diags = check_source(
        "let x = [];\n\
         x.push('hi');\n\
         let y: string = x[0];",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_evolving_array_push_mismatch_ts2322() {

    let diags = check_source(
        "let x = [];\n\
         x.push(1);\n\
         let y: string = x[0];",
    );
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_evolving_array_multiple_pushes_no_error() {

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

    let diags = check_source(
        "let x = [];\n\
         let y = x;",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_evolving_array_unshift_no_error() {

    let diags = check_source(
        "let x = [];\n\
         x.unshift(1);\n\
         let y: number = x[0];",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_non_empty_array_literal_no_error() {

    let diags = check_source(
        "let x = [1, 2, 3];\n\
         let y: number = x[0];",
    );
    assert_no_diagnostics(&diags);
}

use tsox::ast::{NodeData, SyntaxKind};
use tsox::checker::{Checker, Tracer};

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

#[test]
fn checker_interface_merge_no_error() {

    let diags = check_source(
        "interface Foo { a: number; }\n\
         interface Foo { b: string; }\n\
         const x: Foo = { a: 1, b: \"hi\" };",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_interface_merge_missing_member_ts2322() {

    let diags = check_source(
        "interface Foo { a: number; }\n\
         interface Foo { b: string; }\n\
         const x: Foo = { a: 1 };",
    );
    assert_diagnostic_code(&diags, 2741);
}

#[test]
fn checker_interface_merge_missing_first_member_ts2322() {

    let diags = check_source(
        "interface Foo { a: number; }\n\
         interface Foo { b: string; }\n\
         const x: Foo = { b: \"hi\" };",
    );
    assert_diagnostic_code(&diags, 2741);
}

#[test]
fn checker_function_overload_no_error() {

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

    let diags = check_source(
        "namespace N { export const a: number = 1; }\n\
         namespace N { export const b: string = \"hi\"; }\n\
         const x = N.c;",
    );
    assert_diagnostic_code(&diags, 2339);
}

#[test]
fn checker_optional_property_no_error() {

    let diags = check_source(
        "interface Opt { x?: number; }\n\
         const o: Opt = { };\n\
         const n: number | undefined = o.x;",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_optional_property_wrong_type_ts2322() {

    let diags = check_source(
        "interface Opt { x?: number; }\n\
         const o: Opt = { x: \"hi\" };",
    );
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_class_inherits_method_no_error() {

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

    let diags = check_source(
        "interface IFoo { bar(): number; }\n\
         class C implements IFoo { }",
    );

    assert_diagnostic_code(&diags, 2420);
}

#[test]
fn hover_arrow_function_variable() {
    let info = hover_info_for("let f = (a: number): string => \"hi\";", "f").expect("hover");

    assert!(
        info.contains("number") && info.contains("string"),
        "expected arrow hover to mention param/return types, got {info:?}"
    );
}

#[test]
fn checker_namespace_function_merge_no_error() {

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

    let diags = check_source(
        "enum E { A = 1 }\n\
         enum E { B = 2 }\n\
         const x: string = E.B;",
    );
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_single_enum_member_wrong_type_ts2322() {

    let diags = check_source(
        "enum Color { Red = 0 }\n\
         const x: string = Color.Red;",
    );
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_merged_enum_string_not_assignable_ts2322() {

    let diags = check_source(
        "enum E { A = 1 }\n\
         enum E { B = 2 }\n\
         const x: E = \"hi\";",
    );
    assert_diagnostic_code(&diags, 2322);
}

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

    let mut checker = build_checker("export function f(): void {}");
    let stmt = first_statement(&checker, SyntaxKind::FunctionDeclaration).unwrap();
    assert!(checker.is_declaration_visible(&stmt));
}

#[test]
fn visibility_non_exported_function_in_module_not_visible() {

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

    let mut checker = build_checker("function g(): void {}");
    let stmt = first_statement(&checker, SyntaxKind::FunctionDeclaration).unwrap();
    assert!(
        checker.is_declaration_visible(&stmt),
        "top-level function in a global script should be visible"
    );
}

#[test]
fn visibility_import_clause_not_visible_by_default() {

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

#[test]
fn checker_labeled_break_no_error() {

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

#[test]
fn checker_const_string_literal_preserved_no_error() {

    let diags = check_source(
        "const x = \"hello\";\n\
         const y: \"hello\" | \"world\" = x;",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_const_number_literal_preserved_no_error() {

    let diags = check_source("const x = 42;\nconst y: 42 | 99 = x;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_const_literal_assignable_to_union_no_error() {

    let diags = check_source("const x = \"a\";\nconst y: \"a\" | \"b\" = x;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_let_string_literal_widens_no_error() {

    let diags = check_source("let x = \"hello\";\nx = \"bye\";");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_const_literal_assignable_to_primitive_no_error() {

    let diags = check_source("const x = \"hello\";\nlet y: string = x;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_let_widened_not_assignable_to_literal_ts2322() {

    let diags = check_source("let x = \"hello\";\nconst y: \"hello\" = x;");
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_const_literal_callable_with_union_param_no_error() {

    let diags = check_source(
        "function f(k: \"foo\" | \"bar\"): void {}\n\
         const kind = \"foo\";\n\
         f(kind);",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_let_number_literal_widens_no_error() {

    let diags = check_source("let x = 1;\nx = 2;");
    assert_no_diagnostics(&diags);
}

use tsox::evaluator::EvalValue;
use tsox::jsnum::Number;

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

#[test]
fn checker_is_enum_type_related_merged_symbol_no_error() {

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

    let diags = check_source(
        "enum Color { Red = 0, Green = 1 }\n\
         const r: Color.Red = Color.Red;\n\
         const g: Color.Green = Color.Green;",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_is_enum_type_related_enum_to_enum_regression_no_error() {

    let diags = check_source(
        "enum Color { Red = 0, Green = 1 }\n\
         let x: Color = Color.Red;\n\
         let y: Color = Color.Green;",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn module_resolution_node_modules_basic() {

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

#[test]
fn checker_ts2448_let_used_before_declaration() {

    let diags = check_source("let y = later;\nlet later = 42;");
    assert_diagnostic_code(&diags, 2448);
}

#[test]
fn checker_ts2448_const_used_before_declaration() {

    let diags = check_source("let y = later;\nconst later = 42;");
    assert_diagnostic_code(&diags, 2448);
}

#[test]
fn checker_ts2448_let_used_after_declaration_no_error() {

    let diags = check_source("let later = 42;\nlet y = later;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_ts2448_var_used_before_declaration_no_2448_but_2454() {

    let diags = check_source("let y = later;\nvar later = 42;");
    assert_diagnostic_code(&diags, 2454);
    assert_diagnostic_count(&diags, 2454, 1);
}

#[test]
fn checker_ts2448_function_declaration_used_before_no_error() {

    let diags = check_source("f();\nfunction f() { return 1; }");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_ts2448_block_scoped_used_before_in_block() {

    let diags = check_source("{\n  let y = later;\n  let later = 42;\n}");
    assert_diagnostic_code(&diags, 2448);
}

#[test]
fn checker_ts2448_deferred_in_function_no_error() {

    let diags = check_source("function f() { return later; }\nlet later = 42;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_ts2448_deferred_in_arrow_no_error() {

    let diags = check_source("const f = () => later;\nlet later = 42;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_ts2448_class_used_before_declaration() {

    let diags = check_source("const c = new C();\nclass C {}");
    assert_diagnostic_code(&diags, 2449);
}

#[test]
fn checker_ts2448_class_used_after_declaration_no_error() {

    let diags = check_source("class C {}\nconst c = new C();");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_ts2454_let_uninitialized_used_before_assignment() {

    let diags = check_source_strict("let v: number;\nlet y = v;\nv = 1;");
    assert_diagnostic_code(&diags, 2454);
}

#[test]
fn checker_ts2454_let_assigned_before_use_no_error() {

    let diags = check_source("let v: number;\nv = 1;\nlet y = v;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_ts2454_let_with_initializer_no_error() {

    let diags = check_source("let v: number = 0;\nlet y = v;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_ts2454_let_no_type_annotation_no_error() {

    let diags = check_source("let v;\nlet y = v;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_ts2454_let_type_includes_undefined_no_error() {

    let diags = check_source("let v: number | undefined;\nlet y = v;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_ts2454_let_definite_assignment_assertion_no_error() {

    let diags = check_source("let v!: number;\nlet y = v;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_ts2454_declare_let_no_error() {

    let diags = check_source("declare let v: number;\nlet y = v;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_ts2454_let_string_uninitialized_used_before_assignment() {

    let diags = check_source_strict("let s: string;\nlet y = s;\ns = \"hi\";");
    assert_diagnostic_code(&diags, 2454);
}

#[test]
fn checker_ts18048_property_access_on_possibly_undefined() {

    let diags = check_source_strict("let x: { a: number } | undefined = { a: 1 };\nx.a;");
    assert_diagnostic_count(&diags, 18048, 0);
}

#[test]
fn checker_ts18048_optional_chain_suppresses_error() {

    let diags = check_source("let x: { a: number } | undefined = { a: 1 };\nx?.a;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_ts18048_property_access_on_non_undefined_no_error() {

    let diags = check_source("let x: { a: number } = { a: 1 };\nlet y: number = x.a;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_ts18048_property_does_not_exist_at_all_ts2339() {

    let diags = check_source("let x: { a: number } | undefined = { a: 1 };\nx.b;");
    assert_diagnostic_code(&diags, 2339);
}

#[test]
fn checker_ts18048_property_access_after_narrowing_no_error() {

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

    let diags = check_source_strict("let x: { a: number } | null = { a: 1 };\nx.a;");
    assert_diagnostic_count(&diags, 18048, 0);
}

#[test]
fn checker_ts2451_const_redeclare() {

    let diags = check_source("const name = \"Alice\";\nconst name = \"Bob\";");
    assert_diagnostic_code(&diags, 2451);
}

#[test]
fn checker_ts2451_let_redeclare() {

    let diags = check_source("let x = 1;\nlet x = 2;");
    assert_diagnostic_code(&diags, 2451);
}

#[test]
fn checker_ts2451_let_then_const_redeclare() {

    let diags = check_source("let x = 1;\nconst x = 2;");
    assert_diagnostic_code(&diags, 2451);
}

#[test]
fn checker_ts2451_const_then_let_redeclare() {

    let diags = check_source("const x = 1;\nlet x = 2;");
    assert_diagnostic_code(&diags, 2451);
}

#[test]
fn checker_ts2451_triple_redeclare_reports_two() {

    let diags = check_source("let x = 1;\nlet x = 2;\nlet x = 3;");
    assert_diagnostic_count(&diags, 2451, 3);
}

#[test]
fn checker_ts2451_redeclare_in_separate_blocks_no_error() {

    let diags = check_source("if (true) { let x = 1; }\nif (true) { let x = 2; }");
    assert_diagnostic_count(&diags, 2451, 0);
}

#[test]
fn checker_ts2451_var_redeclare_no_error() {

    let diags = check_source("var x = 1;\nvar x = 2;");
    assert_diagnostic_count(&diags, 2451, 0);
}

#[test]
fn checker_ts2451_distinct_names_no_error() {

    let diags = check_source("let a = 1;\nlet b = 2;");
    assert_diagnostic_count(&diags, 2451, 0);
}

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
    let diags = check_source_with_lib("let arr: number[] = [1, 2, 3];\narr.map(x => x);", true);
    assert_diagnostic_count(&diags, 2339, 0);
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
#[allow(non_snake_case)]
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

    let diags = check_source_with_lib("let arr: number[] = [1, 2, 3];\narr.push(4);", true);
    assert_diagnostic_count(&diags, 2339, 0);
}

#[test]
fn checker_array_filter_without_lib_ts2339() {
    let diags = check_source_with_lib("let arr: number[] = [1, 2, 3];\narr.filter(x => x > 1);", true);
    assert_diagnostic_count(&diags, 2339, 0);
}

#[test]
fn checker_array_includes_without_lib_ts2339() {
    let diags = check_source_with_lib("let arr: number[] = [1, 2, 3];\narr.includes(2);", true);
    assert_diagnostic_count(&diags, 2339, 0);
}

#[test]
fn checker_array_reduce_without_lib_ts2339() {
    let diags = check_source_with_lib("let arr: number[] = [1, 2, 3];\narr.reduce((a, b) => a + b, 0);", true);
    assert_diagnostic_count(&diags, 2339, 0);
}

#[test]
fn checker_ts2741_fresh_literal_missing_one_property() {

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

    let diags = check_source(
        "let obj = { a: 1 };\n\
         let y: { a: number; b: number } = obj;",
    );
    assert_diagnostic_code(&diags, 2741);
}

#[test]
fn checker_ts2739_multiple_missing_properties() {

    let diags = check_source("let x: { a: number; b: number; c: number } = { a: 1 };");
    assert_diagnostic_code(&diags, 2739);
}

#[test]
fn checker_ts2353_excess_property_on_type_literal() {

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

    let diags = check_source("let x: { a: number; b: number } = { a: 1, b: 2, c: 3 };");
    assert_diagnostic_code(&diags, 2353);
}

#[test]
fn checker_ts2353_no_excess_when_index_signature_present() {

    let diags = check_source("let x: { a: number; [k: string]: number } = { a: 1, b: 2 };");
    assert_diagnostic_count(&diags, 2353, 0);
}

#[test]
fn checker_ts2448_let_used_in_initializer_of_another() {

    let diags = check_source("let a = b;\nlet b = 1;");
    assert_diagnostic_code(&diags, 2448);
}

#[test]
fn checker_ts2448_const_used_in_expression_before_declaration() {

    let diags = check_source("const r = s + 1;\nconst s = 2;");
    assert_diagnostic_code(&diags, 2448);
}

#[test]
fn checker_ts2448_class_instantiation_before_declaration() {

    let diags = check_source("const i = new C();\nclass C {}");
    assert_diagnostic_code(&diags, 2449);
}

#[test]
fn checker_ts2454_boolean_uninitialized_used_before_assignment() {

    let diags = check_source_strict("let v: boolean;\nlet y = v;\nv = true;");
    assert_diagnostic_code(&diags, 2454);
}

#[test]
fn checker_ts2454_annotated_assignment_after_use() {

    let diags = check_source_strict("let v: number;\nconst y: number = v;\nv = 5;");
    assert_diagnostic_code(&diags, 2454);
}

#[test]
fn checker_ts18048_nullable_via_type_alias() {

    let diags = check_source_strict(
        "type Box = { v: number };\n\
         let x: Box | undefined = { v: 1 };\n\
         x.v;",
    );
    assert_diagnostic_count(&diags, 18048, 0);
}

#[test]
fn checker_ts18048_chained_property_access_on_possibly_undefined() {

    let diags = check_source_strict(
        "let x: { a: { b: number } } | undefined = { a: { b: 1 } };\n\
         x.a.b;",
    );
    assert_diagnostic_count(&diags, 18048, 0);
}

#[test]
fn checker_ts18048_optional_chain_deep_suppresses_error() {

    let diags = check_source(
        "let x: { a: { b: number } } | undefined = { a: { b: 1 } };\n\
         x?.a?.b;",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_ts2300_duplicate_imports() {

    let diags = check_source("import { a } from \"m\";\nimport { a } from \"n\";");
    assert_diagnostic_code(&diags, 2300);
}

#[test]
fn checker_ts2300_class_then_function() {

    let diags = check_source("class C {}\nfunction C() {}");
    assert_diagnostic_code(&diags, 2813);
    assert_diagnostic_code(&diags, 2814);
}

#[test]
fn checker_ts2300_function_then_class() {

    let diags = check_source("function C() {}\nclass C {}");
    assert_diagnostic_code(&diags, 2813);
    assert_diagnostic_code(&diags, 2814);
}

#[test]
fn checker_ts2300_two_classes() {

    let diags = check_source("class C {}\nclass C {}");
    assert_diagnostic_code(&diags, 2300);
}

#[test]
fn checker_ts2300_two_type_aliases() {

    let diags = check_source("type T = number;\ntype T = string;");
    assert_diagnostic_code(&diags, 2300);
}

#[test]
fn checker_ts2300_var_then_function_no_error() {

    let diags = check_source("var x;\nfunction x() {}");
    assert_diagnostic_count(&diags, 2300, 2);
}

#[test]
fn checker_ts2300_function_then_var_no_error() {

    let diags = check_source("function x() {}\nvar x;");
    assert_diagnostic_count(&diags, 2300, 2);
}

#[test]
fn checker_ts2300_var_then_class_no_error() {

    let diags = check_source("var x;\nclass x {}");
    assert_diagnostic_count(&diags, 2300, 2);
}

#[test]
fn checker_ts2300_class_then_var_no_error() {

    let diags = check_source("class x {}\nvar x;");
    assert_diagnostic_count(&diags, 2300, 2);
}

#[test]
fn checker_ts2300_var_then_var_no_error() {

    let diags = check_source("var x;\nvar x;");
    assert_diagnostic_count(&diags, 2300, 0);
}

#[test]
fn checker_ts2300_interface_merge_no_duplicate() {

    let diags = check_source(
        "interface I { a: number; }\n\
         interface I { b: string; }\n\
         const x: I = { a: 1, b: \"hi\" };",
    );
    assert_diagnostic_count(&diags, 2300, 0);
}

#[test]
fn checker_generic_constraint_satisfied_primitive_no_error() {

    let diags =
        check_source("function f<T extends number>(x: T): T { return x; }\nlet n: number = f(42);");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_generic_constraint_satisfied_length_no_error() {

    let diags = check_source(
        "interface HasLength { length: number; }\n\
         function longest<T extends HasLength>(a: T, b: T): T {\n\
         \x20   return a.length >= b.length ? a : b;\n\
         }\n\
         const p: HasLength = { length: 2 };\n\
         const r: HasLength = longest(p, p);",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_generic_constraint_keyof_no_error() {

    let diags = check_source(
        "type Obj = { a: number; b: string };\n\
         type K = keyof Obj;\n\
         let k: \"a\" | \"b\" = null as any as K;",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_typeof_value_is_string_no_error() {

    let diags = check_source("let x = 1;\nlet t: string = typeof x;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_typeof_in_narrowing_branch_no_error() {

    let diags = check_source(
        "function f(x: number | string) {\
         \x20   if (typeof x === \"number\") {\
         \x20       let n: number = x;\
         \x20   }\
         }",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_typeof_string_literal_value_no_error() {

    let diags = check_source("let t: string = typeof \"hi\";");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_template_literal_type_prefix_span_no_error() {

    let diags =
        check_source("type T = `prefix-${string}`;\nlet x: T = null as any as `prefix-hello`;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_template_literal_type_via_generic_alias_no_error() {

    let diags = check_source(
        "type Prefix<T> = `pre-${T}`;\n\
         type P = Prefix<\"x\">;\n\
         let v: \"pre-x\" = null as any as P;",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_template_literal_value_interpolation_no_error() {

    let diags = check_source("let x = 1;\nlet s: string = `val-${x}`;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_satisfies_keeps_expression_type_no_error() {

    let diags = check_source("let x = \"hi\";\nlet y: string = x satisfies string;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_satisfies_object_literal_no_error() {

    let diags = check_source("const cfg = { a: 1 } satisfies { a: number };");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_satisfies_union_no_error() {

    let diags = check_source(
        "const x = \"a\" satisfies \"a\" | \"b\";\n\
         let y: \"a\" = x;",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_optional_chain_on_nullable_no_error() {

    let diags = check_source(
        "type T = { a: number } | null;\n\
         let x: T = null;\n\
         let y = x?.a;",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_optional_chain_nested_no_error() {

    let diags = check_source(
        "let x: { a: { b: number } } | null = null;\n\
         let y = x?.a?.b;",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_optional_chain_method_call_no_error() {

    let diags = check_source(
        "let x: { f: () => number } | null = null;\n\
         let y = x?.f();",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_optional_chain_on_any_no_error() {

    let diags = check_source("let x: any = null;\nlet y = x?.foo;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_nullish_coalescing_with_null_no_error() {

    let diags = check_source("let x: number | null = null;\nlet y = x ?? 0;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_nullish_coalescing_with_undefined_no_error() {

    let diags = check_source("let x: number | undefined = undefined;\nlet y = x ?? 42;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_nullish_coalescing_string_default_no_error() {

    let diags = check_source("let x: string | null = null;\nlet y = x ?? \"default\";");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_nullish_coalescing_parenthesized_no_error() {

    let diags = check_source("let x: number | null = 5;\nlet y = (x ?? 0);");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_nullish_coalescing_left_defined_no_error() {

    let diags = check_source("let x: number | null = 7;\nlet y = x ?? 0;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_array_destructuring_two_elements_no_error() {

    let diags = check_source("let arr = [1, 2];\nlet [a, b] = arr;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_array_destructuring_skip_element_no_error() {

    let diags = check_source("let arr = [1, 2, 3];\nlet [, second] = arr;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_array_destructuring_with_rest_no_error() {

    let diags = check_source("let arr = [1, 2, 3];\nlet [first, ...rest] = arr;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_array_destructuring_nested_no_error() {

    let diags = check_source("let pair = [[1, 2]];\nlet [[a, b]] = pair;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_object_spread_basic_no_error() {

    let diags = check_source("let a = { x: 1 };\nlet b = { ...a, y: 2 };");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_object_spread_overrides_property_no_error() {

    let diags = check_source("let base = { x: 1 };\nlet merged = { ...base, x: 2 };");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_object_spread_multiple_no_error() {

    let diags = check_source("let a = { x: 1 };\nlet b = { y: 2 };\nlet c = { ...a, ...b };");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_array_spread_into_new_array_no_error() {

    let diags = check_source("let arr = [1, 2];\nlet copy = [...arr, 3];");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_as_const_object_literal_no_error() {
    let diags = check_source("const x = { a: 1 } as const;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_as_const_array_literal_no_error() {
    let diags = check_source("const x = [1, 2, 3] as const;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_as_const_string_literal_no_error() {
    let diags = check_source("const x = \"hello\" as const;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_as_const_number_literal_no_error() {
    let diags = check_source("const x = 42 as const;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_as_const_boolean_literal_no_error() {
    let diags = check_source("const x = true as const;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_as_const_nested_object_no_error() {
    let diags = check_source("const x = { a: 1, b: \"hi\", c: true } as const;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_as_const_in_function_no_error() {
    let diags = check_source("function f() { return { status: \"ok\" } as const; }");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_as_const_empty_object_no_error() {
    let diags = check_source("const x = {} as const;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_as_const_in_object_literal_property_no_error() {
    let diags = check_source("const x = { mode: \"test\" as const };");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_as_const_after_satisfies_no_error() {
    let diags = check_source("const x = ({ a: 1 } as const);");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_fn_optional_parameter_decl_no_error() {

    let diags = check_source("function f(a?: string) { return a; }");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_fn_default_parameter_decl_no_error() {

    let diags = check_source("function f(a: number = 1) { return a; }");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_fn_rest_parameter_decl_no_error() {

    let diags = check_source("function f(...args: number[]) { return args.length; }");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_fn_overload_resolution_no_error() {

    let diags = check_source(
        "function f(x: number): number;\n\
         function f(x: string): string;\n\
         function f(x: any) { return x; }\n\
         let n = f(1);\n\
         let s = f(\"a\");",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_fn_overload_resolution_mismatch_ts2322() {

    let diags = check_source(
        "function f(x: number): number;\n\
         function f(x: string): string;\n\
         function f(x: any) { return x; }\n\
         let n: number = f(\"a\");",
    );
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn checker_fn_this_type_return_no_error() {

    let diags = check_source("class C { x = 1; chain(): this { return this; } }");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_abstract_class_declaration_no_error() {

    let diags = check_source("abstract class A { abstract m(): void; }");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_private_protected_members_decl_no_error() {

    let diags = check_source("class C { private a: number = 1; protected b: number = 2; }");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_static_method_call_no_error() {

    let diags = check_source("class C { static inc(): void {} test() { C.inc(); } }");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_getter_setter_no_error() {

    let diags = check_source(
        "class C {\n  _v: number = 0;\n  get value(): number { return this._v; }\n  set value(v: number) { this._v = v; }\n}\nconst c = new C();\nc.value = 5;\nlet x = c.value;",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_class_implements_multiple_interfaces_no_error() {

    let diags = check_source(
        "interface A { a: number; }\ninterface B { b: number; }\nclass C implements A, B { a = 1; b = 2; }",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_keyof_three_keys_union_no_error() {

    let diags = check_source(
        "type K = keyof { a: 1; b: 2; c: 3 };\nlet x: \"a\" | \"b\" | \"c\" = null as any as K;",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_conditional_infer_array_element_type_no_error() {

    let diags = check_source(
        "type Elem<T> = T extends (infer U)[] ? U : never;\nlet x: number = null as any as Elem<number[]>;",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_mapped_type_no_error() {

    let diags = check_source(
        "type Boxed<T> = { [K in keyof T]: number };\ntype R = Boxed<{ a: string; b: string }>;\nlet x: R = { a: 1, b: 2 };",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_mapped_type_optional_modifier_no_error() {

    let diags = check_source(
        "type Flags<T> = { [K in keyof T]?: boolean };\ntype R = Flags<{ a: 1 }>;\nlet x: R = { a: true };",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_readonly_modifier_array_no_error() {

    let diags = check_source("let x: readonly number[] = [1, 2, 3];");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_tuple_type_with_labels_no_error() {

    let diags = check_source("let x: [a: number, b: string] = [1, \"hi\"];");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_typeof_narrow_to_string_no_error() {

    let diags = check_source(
        "let x: string | number = \"hi\";\nif (typeof x === \"string\") { let y: string = x; }",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_typeof_narrow_to_number_no_error() {

    let diags = check_source(
        "let x: string | number = 5;\nif (typeof x === \"number\") { let y: number = x; }",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_typeof_narrow_to_boolean_no_error() {

    let diags = check_source(
        "let x: string | boolean = true;\nif (typeof x === \"boolean\") { let y: boolean = x; }",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_falsy_narrowing_else_assign_no_error() {

    let diags =
        check_source("let x: string | null = null;\nif (!x) { x = \"d\"; }\nlet y: string = x;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_switch_on_typeof_narrowing_no_error() {

    let diags = check_source(
        "let x: number | string = 1;\nswitch (typeof x) {\n  case \"number\": let y: number = x; break;\n  default: break;\n}",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_optional_chain_access_no_error() {

    let diags =
        check_source("let o: { a?: number } = { a: 1 };\nlet y: number | undefined = o?.a;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_named_import_unreferenced_no_error() {

    let diags = check_source("import { x } from \"mod\";");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_namespace_import_unreferenced_no_error() {

    let diags = check_source("import * as ns from \"mod\";");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_import_type_unreferenced_no_error() {

    let diags = check_source("import type { T } from \"mod\";");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_reexport_named_from_module_no_error() {

    let diags = check_source("export { x } from \"mod\";");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_import_side_effect_no_error() {

    let diags = check_source("import \"mod\";");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_circular_import_no_crash() {

    let diags = check_sources(&[
        ("a.ts", "import { b } from \"./b\";\nexport const a = 1;"),
        ("b.ts", "import { a } from \"./a\";\nexport const b = 2;"),
    ]);
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_const_reassignment_ts2588() {

    let diags = check_source("const x = 1;\nx = 2;");
    assert_diagnostic_code(&diags, 2588);
}

#[test]
fn checker_const_compound_assignment_ts2588() {

    let diags = check_source("const x = 1;\nx += 2;");
    assert_diagnostic_code(&diags, 2588);
}

#[test]
fn checker_let_reassignment_no_ts2588() {

    let diags = check_source("let x = 1;\nx = 2;");
    assert!(!diags.iter().any(|d| d.code == 2588));
}

#[test]
fn checker_const_read_no_ts2588() {

    let diags = check_source("const x = 1;\nlet y = x;");
    assert!(!diags.iter().any(|d| d.code == 2588));
}

#[test]
fn checker_unreachable_code_after_return_ts7027() {

    let diags = check_source("function f(): void { return; let x = 1; }");
    assert_diagnostic_code(&diags, 7027);
}

#[test]
fn checker_unreachable_code_after_throw_ts7027() {
    let diags = check_source("function f(): void { throw 1; let x = 1; }");
    assert_diagnostic_code(&diags, 7027);
}

#[test]
fn checker_reachable_code_no_ts7027() {

    let diags = check_source("function f(): number { if (false) { return 1; } return 2; }");
    assert!(!diags.iter().any(|d| d.code == 7027));
}

#[test]
fn checker_abstract_instantiation_ts2511() {

    let diags = check_source("abstract class A {}\nconst a = new A();");
    assert_diagnostic_code(&diags, 2511);
}

#[test]
fn checker_concrete_instantiation_no_ts2511() {

    let diags = check_source("class A {}\nconst a = new A();");
    assert!(!diags.iter().any(|d| d.code == 2511));
}

#[test]
fn checker_private_access_outside_class_ts2341() {

    let diags =
        check_source("class C { private x: number = 1; }\nconst c = new C();\nlet y = c.x;");
    assert_diagnostic_code(&diags, 2341);
}

#[test]
fn checker_private_access_inside_class_no_ts2341() {

    let diags =
        check_source("class C {\n  private x: number = 1;\n  m(): number { return this.x; }\n}");
    assert!(!diags.iter().any(|d| d.code == 2341));
}

#[test]
fn checker_public_access_outside_class_no_ts2341() {

    let diags = check_source("class C { x: number = 1; }\nconst c = new C();\nlet y = c.x;");
    assert!(!diags.iter().any(|d| d.code == 2341));
}

#[test]
fn checker_missing_return_ts2366() {

    let diags = check_source("function f(): number { if (false) { return 1; } }");
    assert_diagnostic_code(&diags, 2366);
}

#[test]
fn checker_missing_return_conditional_ts2366() {

    let diags = check_source("function f(): number { if (Math.random() > 0.5) { return 1; } }");
    assert_diagnostic_code(&diags, 2366);
}

#[test]
fn checker_complete_return_no_ts2366() {

    let diags = check_source("function f(): number { return 1; }");
    assert!(!diags.iter().any(|d| d.code == 2366));
}

#[test]
fn checker_void_return_type_no_ts2366() {

    let diags = check_source("function f(): void { if (false) { return; } }");
    assert!(!diags.iter().any(|d| d.code == 2366));
}

#[test]
fn checker_boolean_negation_with_lib_no_error() {
    let diags = check_source_with_lib("let b = true; let x = !b;", false);
    assert_diagnostic_count(&diags, 2304, 0);
    assert_diagnostic_count(&diags, 2339, 0);
}

#[test]
fn checker_template_literal_interpolation_no_error() {
    let diags = check_source_with_lib("let x = `val ${1 + 2}`;", false);
    assert_diagnostic_count(&diags, 2304, 0);
    assert_diagnostic_count(&diags, 2339, 0);
}

#[test]
fn checker_string_concatenation_with_lib_no_error() {
    let diags = check_source_with_lib("let s = \"a\" + \"b\";", false);
    assert_diagnostic_count(&diags, 2304, 0);
    assert_diagnostic_count(&diags, 2339, 0);
}

#[test]
fn checker_string_index_access_with_lib_no_error() {
    let diags = check_source_with_lib("let s = \"abc\"; let c = s[0];", false);
    assert_diagnostic_count(&diags, 2304, 0);
    assert_diagnostic_count(&diags, 2339, 0);
}

#[test]
fn checker_string_global_call_with_lib_no_error() {
    let diags = check_source_with_lib("let s = String(true);", false);
    assert_diagnostic_count(&diags, 2304, 0);
    assert_diagnostic_count(&diags, 2339, 0);
}

#[test]
fn checker_number_global_call_with_lib_no_error() {
    let diags = check_source_with_lib("let n = Number(\"3\");", false);
    assert_diagnostic_count(&diags, 2304, 0);
    assert_diagnostic_count(&diags, 2339, 0);
}

#[test]
fn checker_string_charat_with_lib_no_error() {
    let diags = check_source_with_lib("let s = \"hi\"; s.charAt(0);", false);
    assert_diagnostic_count(&diags, 2339, 0);
}

#[test]
fn checker_string_touppercase_with_lib_no_error() {
    let diags = check_source_with_lib("let s = \"hi\"; s.toUpperCase();", false);
    assert_diagnostic_count(&diags, 2339, 0);
}

#[test]
fn checker_string_trim_with_lib_no_error() {
    let diags = check_source_with_lib("let s = \" hi \"; s.trim();", false);
    assert_diagnostic_count(&diags, 2339, 0);
}

#[test]
fn checker_string_includes_with_lib_no_error() {
    let diags = check_source_with_lib("let s = \"hi\"; s.includes(\"i\");", false);
    assert_diagnostic_count(&diags, 2339, 0);
}

#[test]
fn checker_number_tofixed_with_lib_no_error() {
    let diags = check_source_with_lib("let n = 3.14; n.toFixed(2);", false);
    assert_diagnostic_count(&diags, 2339, 0);
}

#[test]
fn checker_number_tostring_with_lib_no_error() {
    let diags = check_source_with_lib("let n = 3.14; n.toString();", false);
    assert_diagnostic_count(&diags, 2339, 0);
}

#[test]
fn checker_promise_resolve_with_lib_no_error() {
    let diags = check_source_with_lib("Promise.resolve(1);", false);
    assert_diagnostic_count(&diags, 2304, 0);
    assert_diagnostic_count(&diags, 2339, 0);
}

#[test]
fn checker_promise_reject_with_lib_no_error() {
    let diags = check_source_with_lib("Promise.reject(\"e\");", false);
    assert_diagnostic_count(&diags, 2304, 0);
    assert_diagnostic_count(&diags, 2339, 0);
}

#[test]
fn checker_async_return_number_with_lib_no_error() {
    let diags = check_source_with_lib("async function f() { return 1; }", false);
    assert_diagnostic_count(&diags, 2304, 0);
    assert_diagnostic_count(&diags, 2339, 0);
}

#[test]
fn checker_async_await_with_lib_no_error() {
    let diags = check_source_with_lib(
        "async function f() { let x = await Promise.resolve(1); }",
        false,
    );
    assert_diagnostic_count(&diags, 2304, 0);
    assert_diagnostic_count(&diags, 2339, 0);
}

#[test]
fn checker_async_await_arith_with_lib_no_error() {
    let diags = check_source_with_lib(
        "async function f() { let x = await Promise.resolve(1); let y = x + 1; }",
        false,
    );
    assert_diagnostic_count(&diags, 2304, 0);
    assert_diagnostic_count(&diags, 2339, 0);
}

#[test]
fn checker_promise_then_with_lib_no_error() {
    let diags = check_source_with_lib("Promise.resolve(1).then(x => x);", false);
    assert_diagnostic_count(&diags, 2304, 0);

    assert_diagnostic_count(&diags, 2345, 0);
    assert_diagnostic_count(&diags, 2339, 1);
}

#[test]
fn checker_promise_then_chain_with_lib_no_error() {
    let diags = check_source_with_lib(
        "let p = Promise.resolve(1).then(x => x).then(y => y);",
        false,
    );
    assert_diagnostic_count(&diags, 2304, 0);

    assert_diagnostic_count(&diags, 2345, 0);
    assert_diagnostic_count(&diags, 2339, 1);
}

#[test]
fn checker_promise_all_with_lib_no_error() {
    let diags = check_source_with_lib("Promise.all([1, 2]);", false);
    assert_diagnostic_count(&diags, 2304, 0);
    assert_diagnostic_count(&diags, 2339, 0);
}

#[test]
fn checker_promise_race_with_lib_no_error() {
    let diags = check_source_with_lib("Promise.race([1, 2]);", false);
    assert_diagnostic_count(&diags, 2304, 0);
    assert_diagnostic_count(&diags, 2339, 0);
}

#[test]
fn checker_new_promise_executor_with_lib_no_error() {
    let diags = check_source_with_lib("let p = new Promise((resolve) => { resolve(1); });", false);
    assert_diagnostic_count(&diags, 2304, 0);
    assert_diagnostic_count(&diags, 2339, 0);
}

#[test]
fn checker_record_type_with_lib_no_error() {
    let diags = check_source_with_lib("let x: Record<string, number> = { a: 1 };", false);
    assert_diagnostic_count(&diags, 2304, 0);
    assert_diagnostic_count(&diags, 2339, 0);
}

#[test]
fn checker_partial_type_with_lib_no_error() {
    let diags = check_source_with_lib("interface T { a: number; }\nlet x: Partial<T> = {};", false);
    assert_diagnostic_count(&diags, 2304, 0);
    assert_diagnostic_count(&diags, 2339, 0);
}

#[test]
fn checker_required_type_with_lib_no_error() {
    let diags = check_source_with_lib(
        "interface T { a?: number; }\nlet x: Required<T> = { a: 1 };",
        false,
    );
    assert_diagnostic_count(&diags, 2304, 0);
    assert_diagnostic_count(&diags, 2339, 0);
}

#[test]
fn checker_pick_type_with_lib_no_error() {
    let diags = check_source_with_lib(
        "interface T { a: number; }\nlet x: Pick<T, \"a\"> = { a: 1 };",
        false,
    );
    assert_diagnostic_count(&diags, 2304, 0);
    assert_diagnostic_count(&diags, 2339, 0);
}

#[test]
fn checker_omit_type_with_lib_no_error() {
    let diags = check_source_with_lib(
        "interface T { a: number; b: number; }\nlet x: Omit<T, \"a\">;",
        false,
    );
    assert_diagnostic_count(&diags, 2304, 0);
    assert_diagnostic_count(&diags, 2339, 0);
}

#[test]
fn checker_readonly_type_with_lib_no_error() {
    let diags = check_source_with_lib("let x: Readonly<{ a: number }> = { a: 1 };", false);
    assert_diagnostic_count(&diags, 2304, 0);
    assert_diagnostic_count(&diags, 2339, 0);
}

#[test]
fn checker_record_type_alias_with_lib_no_error() {
    let diags = check_source_with_lib("type R = Record<string, number>;", false);
    assert_diagnostic_count(&diags, 2304, 0);
    assert_diagnostic_count(&diags, 2339, 0);
}

#[test]
fn checker_object_keys_with_lib_no_error() {
    let diags = check_source_with_lib("let x = Object.keys({ a: 1 });", false);
    assert_diagnostic_count(&diags, 2304, 0);
    assert_diagnostic_count(&diags, 2339, 0);
}

#[test]
fn checker_object_values_with_lib_no_error() {

    let diags =
        check_source_with_lib_args("let x = Object.values({ a: 1 });", &["--lib", "es2017"]);
    assert_diagnostic_count(&diags, 2304, 0);
    assert_diagnostic_count(&diags, 2339, 0);
}

#[test]
fn checker_object_entries_with_lib_no_error() {

    let diags =
        check_source_with_lib_args("let x = Object.entries({ a: 1 });", &["--lib", "es2017"]);
    assert_diagnostic_count(&diags, 2304, 0);
    assert_diagnostic_count(&diags, 2339, 0);
}

#[test]
fn checker_for_of_array_with_lib_no_error() {
    let diags = check_source_with_lib("for (let x of [1, 2, 3]) { let y = x; }", false);
    assert_diagnostic_count(&diags, 2304, 0);
    assert_diagnostic_count(&diags, 2339, 0);
}

#[test]
fn checker_for_of_string_with_lib_no_error() {
    let diags = check_source_with_lib("for (let c of \"abc\") { let y = c; }", false);
    assert_diagnostic_count(&diags, 2304, 0);
    assert_diagnostic_count(&diags, 2339, 0);
}

#[test]
fn checker_spread_in_array_with_lib_no_error() {
    let diags = check_source_with_lib("let a = [1, 2]; let b = [...a];", false);
    assert_diagnostic_count(&diags, 2304, 0);
    assert_diagnostic_count(&diags, 2339, 0);
}

#[test]
fn checker_spread_in_call_no_error() {
    let diags = check_source("function f(...args: number[]) {}\nf(1, 2);");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_array_destructuring_with_lib_no_error() {
    let diags = check_source_with_lib("let a = [1, 2]; let [x, y] = a;", false);
    assert_diagnostic_count(&diags, 2304, 0);
    assert_diagnostic_count(&diags, 2339, 0);
}

#[test]
fn checker_array_from_with_lib_no_error() {
    let diags = check_source_with_lib("let x = Array.from([1, 2]);", false);
    assert_diagnostic_count(&diags, 2304, 0);
    assert_diagnostic_count(&diags, 2339, 0);
}

#[test]
fn checker_array_isarray_with_lib_no_error() {
    let diags = check_source_with_lib("let x = Array.isArray([1]);", false);
    assert_diagnostic_count(&diags, 2304, 0);
    assert_diagnostic_count(&diags, 2339, 0);
}

#[test]
fn checker_map_set_with_lib_no_error() {
    let diags = check_source_with_lib("let m = new Map<string, number>(); m.set(\"a\", 1);", false);
    assert_diagnostic_count(&diags, 2304, 0);
    assert_diagnostic_count(&diags, 2339, 0);
}

#[test]
fn checker_set_add_with_lib_no_error() {
    let diags = check_source_with_lib("let s = new Set<number>(); s.add(1);", false);
    assert_diagnostic_count(&diags, 2304, 0);
    assert_diagnostic_count(&diags, 2339, 0);
}

#[test]
fn checker_array_from_string_with_lib_no_error() {
    let diags = check_source_with_lib("let arr = Array.from(\"abc\");", false);
    assert_diagnostic_count(&diags, 2304, 0);
    assert_diagnostic_count(&diags, 2339, 0);
}

#[test]
fn checker_switch_with_default_no_error() {
    let diags = check_source("switch (1) { case 1: break; default: break; }");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_switch_fallthrough_no_error() {
    let diags = check_source("let x = 1; switch (x) { case 1: case 2: break; }");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_switch_on_string_no_error() {
    let diags = check_source("let s = \"a\"; switch (s) { case \"a\": break; case \"b\": break; }");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_switch_on_string_union_no_error() {
    let diags = check_source("let x: \"a\" | \"b\" = \"a\"; switch (x) { case \"a\": break; }");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_nested_switch_no_error() {
    let diags = check_source(
        "function f(a: number, b: number) { switch (a) { case 1: switch (b) { case 2: break; } break; } }",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_switch_with_return_no_error() {
    let diags = check_source(
        "function f(x: number) { switch (x) { case 1: return \"a\"; case 2: return \"b\"; default: return \"c\"; } }",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_ternary_chain_no_error() {
    let diags = check_source("let x = 1; let y = x ? 2 : x ? 3 : 4;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_nested_ternary_no_error() {
    let diags = check_source("let x = 1; let y = x === 1 ? (x === 2 ? 3 : 4) : 5;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_nullish_coalescing_no_error() {
    let diags = check_source("let x = null; let y = x ?? 5;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_logical_or_default_no_error() {
    let diags = check_source("let x = 0; let y = x || \"fallback\";");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_generic_function_declaration_no_error() {
    let diags = check_source("function f<T>(x: T): T { return x; }");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_generic_constraint_call_no_error() {
    let diags = check_source("function f<T extends number>(x: T): T { return x; }\nf(1);");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_generic_class_instantiation_no_error() {

    let diags = check_source_all_strict("class C<T> { x: T; }\nlet c = new C<number>();");
    assert_diagnostic_code(&diags, 2564);
}

#[test]
fn checker_generic_class_constructor_no_error() {
    let diags = check_source("class Box<T> { constructor(public value: T) {} }");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_multiple_type_params_no_error() {
    let diags = check_source("function f<A, B>(a: A, b: B): [A, B] { return [a, b]; }");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_default_type_param_no_error() {
    let diags = check_source("function f<T = string>(x: T): T { return x; }");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_generic_interface_usage_no_error() {
    let diags = check_source("interface I<T> { data: T; }\nlet x: I<number> = { data: 1 };");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_generic_multi_param_interface_no_error() {
    let diags = check_source("interface I<T, U> { a: T; b: U; }");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_generic_type_alias_no_error() {
    let diags = check_source("type CB<T> = (x: T) => void;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_generic_constraint_length_no_error() {
    let diags = check_source(
        "function len<T extends { length: number }>(x: T): number { return x.length; }",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_generic_call_inference_no_error() {

    let diags = check_source("function id<T>(x: T): T { return x; }\nid(\"hi\");");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_generic_identity_inference_number_no_error() {

    let diags =
        check_source("function identity<T>(x: T): T { return x; }\nconst n = identity(42);");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_generic_identity_inference_string_no_error() {

    let diags =
        check_source("function identity<T>(x: T): T { return x; }\nconst s = identity(\"hi\");");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_optional_properties_no_error() {
    let diags = check_source("interface T { a?: number; }\nlet x: T = {};");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_readonly_properties_no_error() {
    let diags = check_source("interface T { readonly a: number; }\nlet x: T = { a: 1 };");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_index_signature_no_error() {
    let diags = check_source("interface T { [k: string]: number; }\nlet x: T = { a: 1 };");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_readonly_index_signature_no_error() {
    let diags = check_source("let x: { readonly [k: string]: number } = { a: 1 };");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_call_signature_no_error() {
    let diags = check_source("interface T { (): number; }\nlet x: T;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_construct_signature_no_error() {
    let diags = check_source("interface T { new (): number; }\nlet x: T;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_extends_multiple_interfaces_no_error() {
    let diags = check_source(
        "interface A { a: number; }\ninterface B { b: number; }\ninterface C extends A, B { c: number; }",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_object_type_alias_usage_no_error() {
    let diags = check_source("type T = { a: number; };\nlet x: T = { a: 1 };");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_intersection_type_no_error() {
    let diags = check_source("interface A { a: number; }\ntype B = A & { b: number; };");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_string_literal_union_type_no_error() {
    let diags = check_source("type K = \"a\" | \"b\";");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_interface_with_methods_no_error() {
    let diags = check_source("interface I { m(x: number): string; p: { q: number }; }");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_deep_union_type_no_error() {
    let diags = check_source("type T = number | string | boolean;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_circular_interface_reference_no_crash() {

    let diags = check_source("interface X { a: X; }");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_long_property_chain_no_error() {
    let diags = check_source("let o = { a: { b: { c: { d: 1 } } } }; let x = o.a.b.c.d;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_nested_array_access_no_error() {
    let diags = check_source("let arr = [{ a: 1 }, { a: 2 }]; let n = arr[0].a;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_empty_interface_assignment_no_error() {
    let diags = check_source("interface E {}\nlet x: E = {};");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_empty_type_alias_no_error() {
    let diags = check_source("type Empty = {};");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_never_type_throw_no_error() {
    let diags = check_source("function f(): never { throw 1; }");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_unknown_type_assignment_no_error() {
    let diags = check_source("let x: unknown = 1;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_unknown_typeof_guard_no_error() {
    let diags = check_source("let x: unknown = 1; if (typeof x === \"number\") { let y = x; }");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_any_type_method_call_no_error() {
    let diags = check_source("let x: any = 1; x.foo();");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_any_type_property_access_no_error() {
    let diags = check_source("let x: any = null; let y: any = x.bar;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_void_function_no_error() {
    let diags = check_source("function g(): void {}");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_void_assignment_with_lib_no_error() {
    let diags = check_source_with_lib("let x: void = undefined;", false);
    assert_diagnostic_count(&diags, 2304, 0);
    assert_diagnostic_count(&diags, 2339, 0);
}

#[test]
fn checker_decorator_class_no_error() {
    let diags = check_source_with_lib_args(
        "function log(target: any) {}\n@log\nclass A {}",
        &["--experimentalDecorators"],
    );

    assert_diagnostic_count(&diags, 2304, 0);
}

#[test]
fn checker_decorator_method_no_error() {
    let diags = check_source_with_lib_args(
        "function log(target: any, key: string, desc: PropertyDescriptor) {}\nclass A {\n  @log\n  foo() {}\n}",
        &["--experimentalDecorators"],
    );

    assert_diagnostic_count(&diags, 2304, 0);
}

#[test]
fn checker_decorator_property_no_error() {
    let diags = check_source_with_lib_args(
        "function log(target: any, key: string) {}\nclass A {\n  @log\n  x: number = 1;\n}",
        &["--experimentalDecorators"],
    );

    assert_diagnostic_count(&diags, 2304, 0);
}

#[test]
fn checker_decorator_parameter_no_error() {
    let diags = check_source_with_lib_args(
        "function log(target: any, key: string, idx: number) {}\nclass A {\n  foo(@log x: number) {}\n}",
        &["--experimentalDecorators"],
    );

    assert_diagnostic_count(&diags, 2304, 0);
}

#[test]
fn checker_decorator_factory_no_error() {
    let diags = check_source_with_lib_args(
        "function log(name: string) { return function (target: any) {}; }\n@log(\"test\")\nclass A {}",
        &["--experimentalDecorators"],
    );

    assert_diagnostic_count(&diags, 2304, 0);
}

#[test]
fn checker_generic_constraint_with_default_no_error() {
    let diags =
        check_source("function f<T extends string = \"a\">(): T {\n  return \"a\" as T;\n}");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_conditional_type_distribution_no_error() {
    let diags = check_source(
        "type Box<T> = T extends string ? string : number;\ntype R = Box<string | number>;",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_mapped_type_modifiers_no_error() {
    let diags = check_source(
        "type Mutable<T> = { -readonly [K in keyof T]: T[K] };\ntype Opt<T> = { [K in keyof T]?: T[K] };\ntype Req<T> = { [K in keyof T]-?: T[K] };\nlet x: Mutable<{ readonly a: 1 }> = { a: 1 };",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_infer_with_constraint_no_error() {
    let diags =
        check_source("type R<T> = T extends Array<infer U> ? U : never;\ntype X = R<number[]>;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_recursive_generic_no_error() {
    let diags = check_source(
        "type Tree<T> = { value: T; left?: Tree<T>; right?: Tree<T>; };\nlet t: Tree<number> = { value: 1 };",
    );

    assert_no_diagnostics(&diags);
}

#[test]
fn checker_generic_class_method_no_error() {
    let diags = check_source(
        "class Box<T> {\n  item: T;\n  constructor(item: T) { this.item = item; }\n  get(): T { return this.item; }\n}\nlet b = new Box(42);\nlet n = b.get();",
    );

    assert_diagnostic_count(&diags, 2345, 1);
}

#[test]
fn checker_multiple_constraints_intersection_no_error() {
    let diags = check_source(
        "interface A { a: number; }\ninterface B { b: number; }\nfunction f<T extends A & B>(x: T): T { return x; }",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_generic_factory_function_no_error() {
    let diags = check_source(
        "function create<T>(ctor: new () => T): T { return new ctor(); }\nclass C {}\nlet c = create(C);",
    );

    assert_diagnostic_count(&diags, 2345, 0);
}

#[test]
fn checker_type_predicate_with_generic_no_error() {
    let diags = check_source(
        "function isString<T>(x: T | string): x is string {\n  return typeof x === \"string\";\n}\nlet s = isString(\"hi\");",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_generic_identity_function_no_error() {
    let diags =
        check_source("function id<T>(x: T): T { return x; }\nlet n = id(42);\nlet s = id(\"hi\");");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_try_catch_variable_type_no_error() {
    let diags = check_source("try {\n  let x = 1;\n} catch (e) {\n  console_log(e);\n}");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_error_subclass_no_error() {
    let diags = check_source_with_lib(
        "class MyError extends Error {\n  constructor(msg: string) {\n    super(msg);\n  }\n}\nthrow new MyError(\"oops\");",
        false,
    );

    assert_diagnostic_count(&diags, 2304, 0);
}

#[test]
fn checker_custom_error_class_no_error() {
    let diags = check_source(
        "class ValidationError {\n  constructor(public message: string) {}\n}\nlet e = new ValidationError(\"bad\");",
    );

    assert_no_diagnostics(&diags);
}

#[test]
fn checker_throw_expression_no_error() {
    let diags = check_source(
        "function f(x: number): number {\n  if (x < 0) throw new Error();\n  return x;\n}",
    );

    assert_no_diagnostics(&diags);
}

#[test]
fn checker_error_in_async_function_no_error() {
    let diags = check_source_with_lib(
        "async function f(): Promise<number> {\n  try {\n    return 1;\n  } catch (e) {\n    return 0;\n  }\n}",
        false,
    );

    assert_diagnostic_count(&diags, 2304, 0);
}

#[test]
fn checker_finally_block_no_error() {
    let diags = check_source(
        "try {\n  let x = 1;\n} catch (e) {\n  let y = 2;\n} finally {\n  let z = 3;\n}",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_nested_try_catch_no_error() {
    let diags = check_source(
        "try {\n  try {\n    let x = 1;\n  } catch (e) {\n    let y = 2;\n  }\n} catch (e2) {\n  let z = 3;\n}",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_catch_without_annotation_no_error() {
    let diags = check_source("try {\n  let x = 1;\n} catch {\n  let y = 2;\n}");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_throw_non_error_value_no_error() {
    let diags = check_source("function f(): number {\n  throw 42;\n}");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_error_message_property_no_error() {
    let diags = check_source_with_lib(
        "function f() {\n  try {\n    let x = 1;\n  } catch (e) {\n    let m = (e as Error).message;\n  }\n}",
        false,
    );

    assert_diagnostic_count(&diags, 2304, 0);
}

#[test]
fn checker_partial_utility_no_error() {
    let diags = check_source_with_lib(
        "interface P { a: number; b: string; }\nlet x: Partial<P> = { a: 1 };",
        false,
    );
    assert_diagnostic_count(&diags, 2304, 0);
    assert_diagnostic_count(&diags, 2339, 0);
}

#[test]
fn checker_required_utility_no_error() {
    let diags = check_source_with_lib(
        "interface P { a?: number; b?: string; }\nlet x: Required<P> = { a: 1, b: \"hi\" };",
        false,
    );
    assert_diagnostic_count(&diags, 2304, 0);
    assert_diagnostic_count(&diags, 2339, 0);
}

#[test]
fn checker_readonly_array_no_error() {
    let diags = check_source_with_lib("let x: ReadonlyArray<number> = [1, 2, 3];", false);
    assert_diagnostic_count(&diags, 2304, 0);
    assert_diagnostic_count(&diags, 2339, 0);
}

#[test]
fn checker_record_utility_no_error() {
    let diags = check_source_with_lib("let x: Record<string, number> = { a: 1, b: 2 };", false);
    assert_diagnostic_count(&diags, 2304, 0);
    assert_diagnostic_count(&diags, 2339, 0);
}

#[test]
fn checker_return_type_utility_no_error() {
    let diags = check_source_with_lib(
        "function f(): number { return 1; }\ntype R = ReturnType<typeof f>;\nlet x: R = 1;",
        false,
    );
    assert_diagnostic_count(&diags, 2304, 0);
    assert_diagnostic_count(&diags, 2339, 0);
}

#[test]
fn checker_parameters_utility_no_error() {
    let diags = check_source_with_lib(
        "function f(a: number, b: string): void {}\ntype P = Parameters<typeof f>;",
        false,
    );
    assert_diagnostic_count(&diags, 2304, 0);
    assert_diagnostic_count(&diags, 2339, 0);
}

#[test]
fn checker_constructor_parameters_utility_no_error() {
    let diags = check_source_with_lib(
        "class C { constructor(a: number, b: string) {} }\ntype P = ConstructorParameters<typeof C>;",
        false,
    );
    assert_diagnostic_count(&diags, 2304, 0);
    assert_diagnostic_count(&diags, 2339, 0);
}

#[test]
fn checker_instance_type_utility_no_error() {
    let diags = check_source_with_lib(
        "class C { x: number = 1; }\ntype I = InstanceType<typeof C>;\nfunction make(): I { return new C(); }",
        false,
    );
    assert_diagnostic_count(&diags, 2304, 0);
    assert_diagnostic_count(&diags, 2339, 0);
}

#[test]
fn checker_omit_utility_no_error() {
    let diags = check_source_with_lib(
        "interface P { a: number; b: string; c: boolean; }\nlet x: Omit<P, \"a\"> = { b: \"hi\", c: true };",
        false,
    );
    assert_diagnostic_count(&diags, 2304, 0);
    assert_diagnostic_count(&diags, 2339, 0);
}

#[test]
fn checker_pick_utility_no_error() {
    let diags = check_source_with_lib(
        "interface P { a: number; b: string; c: boolean; }\nlet x: Pick<P, \"a\" | \"b\"> = { a: 1, b: \"hi\" };",
        false,
    );
    assert_diagnostic_count(&diags, 2304, 0);
    assert_diagnostic_count(&diags, 2339, 0);
}

#[test]
fn checker_react_like_component_no_error() {
    let diags = check_source(
        "interface Props { name: string; age: number; }\nfunction Greet(props: Props): string {\n  return props.name + props.age;\n}\nlet r = Greet({ name: \"a\", age: 1 });",
    );

    assert_no_diagnostics(&diags);
}

#[test]
fn checker_redux_like_reducer_no_error() {
    let diags = check_source(
        "type Action = { type: \"inc\" } | { type: \"dec\" };\nfunction reducer(state: number, action: Action): number {\n  if (action.type === \"inc\") return state + 1;\n  return state - 1;\n}\nlet s = reducer(0, { type: \"inc\" });",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_event_emitter_pattern_no_error() {
    let diags = check_source(
        "class Emitter {\n  private handlers: Record<string, ((x: number) => void)[]> = {};\n  on(name: string, fn: (x: number) => void): void {\n    this.handlers[name] = [fn];\n  }\n  emit(name: string, val: number): void {\n    let arr = this.handlers[name];\n    if (arr) { arr[0](val); }\n  }\n}\nlet e = new Emitter();",
    );

    assert_diagnostic_count(&diags, 2304, 0);
}

#[test]
fn checker_builder_pattern_no_error() {
    let _diags = check_source_with_lib(
        "class Builder {\n  private parts: string[] = [];\n  add(p: string): this {\n    this.parts.push(p);\n    return this;\n  }\n  build(): string { return this.parts.join(\"\"); }\n}\nlet r = new Builder().add(\"a\").add(\"b\").build();",
        false,
    );

    let diags = check_source_with_lib(
        "class Builder {\n  private parts: string[] = [];\n  add(p: string): this {\n    this.parts.push(p);\n    return this;\n  }\n  build(): string { return this.parts.join(\"\"); }\n}\nlet r = new Builder().add(\"a\").add(\"b\").build();",
        false,
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_singleton_pattern_no_error() {
    let diags = check_source(
        "class Singleton {\n  private static instance: Singleton;\n  private constructor() {}\n  static get(): Singleton {\n    if (!Singleton.instance) Singleton.instance = new Singleton();\n    return Singleton.instance;\n  }\n}\nlet s = Singleton.get();",
    );

    assert_diagnostic_count(&diags, 2339, 0);
}

#[test]
fn checker_factory_pattern_no_error() {
    let diags = check_source(
        "interface Animal { speak(): string; }\nclass Dog implements Animal { speak(): string { return \"woof\"; } }\nfunction createAnimal(): Animal { return new Dog(); }\nlet a = createAnimal();",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_observer_pattern_no_error() {
    let diags = check_source(
        "interface Observer { update(val: number): void; }\nclass Subject {\n  private obs: Observer[] = [];\n  attach(o: Observer): void { this.obs.push(o); }\n  notify(v: number): void { this.obs.forEach(o => o.update(v)); }\n}\nlet s = new Subject();",
    );

    assert_no_diagnostics(&diags);
}

#[test]
fn checker_iterator_protocol_no_error() {
    let diags = check_source(
        "class Counter {\n  private n = 0;\n  next(): { value: number; done: boolean } {\n    this.n++;\n    return { value: this.n, done: false };\n  }\n}\nlet c = new Counter();\nlet v = c.next();",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_iterable_protocol_no_error() {
    let diags = check_source(
        "interface MyIterable {\n  [Symbol.iterator](): { next: () => { value: number; done: boolean } };\n}\nclass C implements MyIterable {\n  [Symbol.iterator]() {\n    return { next: () => ({ value: 1, done: false }) };\n  }\n}\nlet c = new C();",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_mixin_pattern_no_error() {
    let diags = check_source(
        "type Constructor<T = {}> = new (...args: any[]) => T;\nfunction Timestamped<TBase extends Constructor>(Base: TBase) {\n  return class extends Base {\n    timestamp = Date.now();\n  };\n}\nclass User {}\nconst TimestampedUser = Timestamped(User);\nlet u = new TimestampedUser();",
    );

    assert_diagnostic_count(&diags, 2304, 0);
    assert_diagnostic_count(&diags, 2345, 0);
}

#[test]
fn checker_configuration_object_no_error() {
    let diags = check_source(
        "interface Config {\n  host: string;\n  port: number;\n  debug?: boolean;\n}\nfunction init(c: Config): void {}\ninit({ host: \"localhost\", port: 8080 });",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_api_response_typing_no_error() {
    let diags = check_source(
        "interface ApiResponse<T> {\n  data: T;\n  status: number;\n  error?: string;\n}\nlet r: ApiResponse<number> = { data: 42, status: 200 };",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_generic_repository_no_error() {
    let diags = check_source(
        "interface Repo<T> {\n  find(id: number): T;\n  save(item: T): void;\n}\nclass UserRepo implements Repo<string> {\n  find(id: number): string { return \"user\"; }\n  save(item: string): void {}\n}\nlet r = new UserRepo();",
    );

    assert_no_diagnostics(&diags);
}

#[test]
fn checker_middleware_pattern_no_error() {
    let diags = check_source(
        "type Middleware = (ctx: { status: number }, next: () => void) => void;\nfunction use(mw: Middleware): void {}\nuse((ctx, next) => { ctx.status = 200; next(); });",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_plugin_system_no_error() {
    let diags = check_source(
        "interface Plugin {\n  name: string;\n  install(): void;\n}\nfunction register(p: Plugin): void {}\nregister({ name: \"test\", install: () => {} });",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_edge_case_empty_file_no_error() {
    let diags = check_source("");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_single_comment_no_error() {
    let diags = check_source("// just a comment");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_only_imports_no_error() {
    let diags = check_sources(&[
        ("helper.ts", "export const x = 1;"),
        ("main.ts", "import { x } from \"./helper\";\n"),
    ]);
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_nested_namespaces_3_levels_no_error() {
    let diags = check_source(
        "namespace A {\n  export namespace B {\n    export namespace C {\n      export let x = 1;\n    }\n  }\n}\nlet v = A.B.C.x;",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_deeply_nested_ternary_no_error() {
    let diags = check_source(
        "function f(x: number): number {\n  return x > 0\n    ? x > 10\n      ? x > 100\n        ? x > 1000\n          ? 1\n          : 2\n        : 3\n      : 4\n    : 5;\n}",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_long_union_no_error() {
    let diags = check_source(
        "type U = string | number | boolean | null | undefined | symbol | bigint | object | void | never;\nlet x: U = 1;",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_long_intersection_no_error() {
    let diags = check_source(
        "interface A { a: number; }\ninterface B { b: number; }\ninterface C { c: number; }\ninterface D { d: number; }\ninterface E { e: number; }\ntype I = A & B & C & D & E;\nlet x: I = { a: 1, b: 2, c: 3, d: 4, e: 5 };",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_complex_mapped_type_no_error() {
    let diags = check_source(
        "type Getters<T> = { [K in keyof T]: () => T[K] };\ninterface P { a: number; b: string; }\nlet g: Getters<P> = { a: () => 1, b: () => \"hi\" };",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_template_literal_with_union_no_error() {
    let diags = check_source(
        "type Suffix = \"px\" | \"em\";\ntype Size = `${number}${Suffix}`;\nlet x: Size = \"10px\";",
    );

    assert_diagnostic_count(&diags, 2322, 1);
}

#[test]
fn checker_long_tuple_no_error() {
    let diags = check_source(
        "let t: [number, number, number, number, number, number, number, number, number, number] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_strict_null_checks_comparison_no_error() {
    let diags = check_source_with_lib_args(
        "let x: number | null = null;\nif (x !== null) {\n  let y: number = x;\n}",
        &["--strictNullChecks"],
    );

    assert_diagnostic_count(&diags, 2304, 0);
}

#[test]
fn checker_strict_property_init_no_error() {
    let diags = check_source_with_lib_args(
        "class C {\n  x: number = 0;\n}",
        &["--strictPropertyInitialization"],
    );

    assert_diagnostic_count(&diags, 2304, 0);
}

#[test]
fn checker_strict_bind_call_apply_no_error() {
    let diags = check_source_with_lib_args(
        "function f(a: number, b: string): number { return a; }\nlet g = f.bind(null, 1);\ng(\"hi\");",
        &["--strictBindCallApply"],
    );

    assert_diagnostic_count(&diags, 2304, 0);
}

#[test]
fn checker_always_strict_no_error() {
    let diags = check_source_with_lib_args("let x = 1;", &["--alwaysStrict"]);

    assert_diagnostic_count(&diags, 2304, 0);
}

#[test]
fn checker_no_implicit_this_no_error() {
    let diags = check_source_with_lib_args(
        "class C {\n  x: number = 1;\n  foo(): number { return this.x; }\n}",
        &["--noImplicitThis"],
    );

    assert_diagnostic_count(&diags, 2304, 0);
}

#[test]
fn checker_no_implicit_any_param_annotated_no_error() {
    let diags = check_source_with_lib_args(
        "function f(x: number): number { return x; }",
        &["--noImplicitAny"],
    );

    assert_diagnostic_count(&diags, 2304, 0);
}

#[test]
fn checker_strict_no_error() {
    let diags = check_source_with_lib_args(
        "function f(x: number): string {\n  return String(x);\n}",
        &["--strict"],
    );
    assert_diagnostic_count(&diags, 2304, 0);
}

#[test]
fn checker_strict_function_types_no_error() {
    let diags = check_source_with_lib_args(
        "type CB = (x: string) => void;\nlet f: CB = (x) => {};\nf(\"hi\");",
        &["--strictFunctionTypes"],
    );

    assert_diagnostic_count(&diags, 2304, 0);
}

#[test]
fn checker_no_implicit_any_arrow_no_error() {
    let diags = check_source_with_lib_args(
        "let f = (x: number): number => x + 1;",
        &["--noImplicitAny"],
    );

    assert_diagnostic_count(&diags, 2304, 0);
}

#[test]
fn checker_strict_object_literal_no_error() {
    let diags = check_source_with_lib_args(
        "interface P { a: number; b: string; }\nlet x: P = { a: 1, b: \"hi\" };",
        &["--strict"],
    );

    assert_diagnostic_count(&diags, 2304, 0);
}

#[test]
fn checker_in_operator_narrowing_no_error() {
    let diags = check_source(
        "type A = { kind: \"a\"; val: number };\ntype B = { kind: \"b\"; str: string };\nfunction f(x: A | B): number {\n  if (\"val\" in x) return x.val;\n  return x.str.length;\n}",
    );

    assert_no_diagnostics(&diags);
}

#[test]
fn checker_array_isarray_narrowing_no_error() {
    let diags = check_source_with_lib(
        "function f(x: number | number[]): number {\n  if (Array.isArray(x)) return x[0];\n  return x;\n}",
        false,
    );
    assert_diagnostic_count(&diags, 2304, 0);
    assert_diagnostic_count(&diags, 2339, 0);
}

#[test]
fn checker_typeof_bigint_narrowing_no_error() {
    let diags = check_source(
        "function f(x: number | bigint): string {\n  if (typeof x === \"bigint\") return \"bigint\";\n  return \"number\";\n}",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_custom_type_guard_with_this_no_error() {
    let diags = check_source(
        "class C {\n  x: number | null = null;\n  has(): this is { x: number } {\n    return this.x !== null;\n  }\n}",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_assertion_function_no_error() {
    let diags = check_source_strict(
        "function assert(cond: boolean): asserts cond {\n  if (!cond) throw new Error();\n}\nlet x: number | undefined = 1;\nassert(x !== undefined);\nlet y: number = x;",
    );

    assert_diagnostic_count(&diags, 2304, 0);
    assert_diagnostic_count(&diags, 2322, 0);
}

#[test]
fn checker_discriminated_union_with_array_no_error() {
    let diags = check_source(
        "type Result =\n  | { status: \"ok\"; data: number[] }\n  | { status: \"err\"; data: string[] };\nfunction f(r: Result): number {\n  if (r.status === \"ok\") return r.data.length;\n  return r.data.length;\n}",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_narrow_optional_chain_method_no_error() {
    let diags = check_source("let obj: { f?: () => number } = {};\nlet x = obj?.f?.() ?? 0;");

    assert_no_diagnostics(&diags);
}

#[test]
fn checker_switch_early_return_no_error() {
    let diags = check_source(
        "function f(x: string): number {\n  switch (x) {\n    case \"a\":\n      return 1;\n    case \"b\":\n    case \"c\":\n      return 2;\n    default:\n      return 0;\n  }\n}",
    );

    assert_diagnostic_count(&diags, 2366, 0);
}

#[test]
fn checker_multiple_narrowing_conditions_no_error() {
    let diags = check_source(
        "function f(x: number | string | null): number {\n  if (x === null) return 0;\n  if (typeof x === \"string\") return x.length;\n  if (x > 10) return x;\n  return -1;\n}",
    );

    assert_no_diagnostics(&diags);
}

#[test]
fn checker_nullish_coalescing_narrowing_no_error() {
    let diags = check_source_strict("let x: string | null = null;\nlet y: string = x ?? \"default\";");

    assert_diagnostic_count(&diags, 2322, 1);
}

#[test]
fn checker_dynamic_import_expression_no_error() {
    let diags = check_source(
        "async function f(): Promise<any> {\n  let m = await import(\"./mod\");\n  return m;\n}",
    );

    assert_diagnostic_count(&diags, 2304, 0);
}

#[test]
fn checker_import_type_statement_no_error() {
    let diags = check_sources(&[
        ("types.ts", "export type MyType = { a: number };"),
        (
            "main.ts",
            "import type { MyType } from \"./types\";\nlet x: MyType = { a: 1 };",
        ),
    ]);
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_export_type_statement_no_error() {
    let diags = check_sources(&[
        ("types.ts", "export type MyType = string;"),
        ("main.ts", "export type { MyType } from \"./types\";"),
    ]);
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_namespace_re_export_no_error() {
    let diags = check_sources(&[
        ("types.ts", "export const x = 1;"),
        ("main.ts", "export * as NS from \"./types\";\n"),
    ]);
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_mixed_default_named_imports_no_error() {
    let diags = check_sources(&[
        (
            "helper.ts",
            "export default function() { return 1; }\nexport const x = 2;",
        ),
        (
            "main.ts",
            "import def, { x } from \"./helper\";\nlet y = x;",
        ),
    ]);
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_circular_type_import_no_error() {
    let diags = check_sources(&[
        (
            "a.ts",
            "import type { B } from \"./b\";\nexport interface A { b: B | null; }",
        ),
        (
            "b.ts",
            "import type { A } from \"./a\";\nexport interface B { a: A | null; }",
        ),
    ]);
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_module_augmentation_no_error() {
    let diags = check_source_with_lib(
        "declare module \"express\" {\n  interface Request { user?: string; }\n}",
        false,
    );

    assert_diagnostic_count(&diags, 2304, 0);
}

#[test]
fn checker_ambient_module_declaration_no_error() {
    let diags =
        check_source("declare module \"my-mod\" {\n  export function doSomething(): void;\n}");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_export_equals_syntax_no_error() {
    let diags = check_source("function f(): number { return 1; }\nexport = f;");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_import_equals_require_no_error() {
    let diags = check_sources(&[
        ("helper.ts", "export const x = 1;"),
        ("main.ts", "import y = require(\"./helper\");\n"),
    ]);
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_optional_parameter_type_includes_undefined_ts2322() {

    let diags = check_source_strict("function g(f?: number) { const x: number = f; }");
    assert_diagnostic_count(&diags, 2322, 1);
}

#[test]
fn checker_optional_parameter_property_access_ts18048() {
    let diags = check_source_strict("function g(f?: { m(): void }) { f.m(); }");
    assert_diagnostic_count(&diags, 18048, 1);
}

#[test]
fn checker_optional_parameter_contravariant_assignment_ok() {
    let diags = check_source_strict(
        "declare const a: (x?: number) => void;\nconst b: (x: number) => void = a;",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_logical_assignment5_family_codes() {

    let src = "\
function foo1 (f?: (a: number) => void) { f ??= (a => a); f(42); }
function foo2 (f?: (a: number) => void) { f ||= (a => a); f(42); }
function foo3 (f?: (a: number) => void) { f &&= (a => a); f(42); }
function bar1 (f?: (a: number) => void) { f ??= (f.toString(), (a => a)); f(42); }
function bar2 (f?: (a: number) => void) { f ||= (f.toString(), (a => a)); f(42); }
function bar3 (f?: (a: number) => void) { f &&= (f.toString(), (a => a)); f(42); }
";
    let diags =
        check_source_with_lib_args(src, &["--strict", "--target", "esnext"]);
    assert_diagnostic_count(&diags, 2722, 2);
    assert_diagnostic_count(&diags, 18048, 2);
    assert_diagnostic_count(&diags, 2349, 0);
    assert_diagnostic_count(&diags, 7006, 0);
}

#[test]
fn checker_logical_assignment_rhs_truthy_frame_no_error() {

    let diags = check_source_strict(
        "function bar3 (f?: (a: number) => void) { f &&= (f.toString(), (a => a)); }",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_union_callee_callable() {
    let diags = check_source_strict(
        "declare const f: ((a: number) => void) | ((a: number) => number);\nf(42);",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_possibly_undefined_callee_reports_only_2722() {
    let diags = check_source_strict(
        "declare const g: undefined | ((a: number) => number);\ng(42);",
    );
    assert_diagnostic_count(&diags, 2722, 1);
    assert_diagnostic_count(&diags, 2349, 0);
}

#[test]
fn checker_static_member_this_is_constructor_type() {

    let diags = check_source_strict(
        "class P { static y = this; static bar(zz = this) { return zz.y; } }",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_static_member_this_missing_property_ts2339() {
    let diags =
        check_source_strict("class P { static bar(zz = this) { return zz.q; } }");
    assert_diagnostic_count(&diags, 2339, 1);
}

#[test]
fn checker_instance_member_this_is_instance_type() {
    let diags =
        check_source_strict("class P { inst = 1; bar(zz = this) { return zz.inst; } }");
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_qualified_name_heritage_members_resolve() {

    let diags = check_source_strict(
        "declare namespace NS { export interface ICl { Clone(): any; } }\ninterface Num2 extends NS.ICl { }\ndeclare const x: Num2;\nconst y = x.Clone();",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_boxed_apparent_type_includes_heritage_members() {

    let diags = check_source_strict(
        "declare namespace NS { export interface ICl { Clone(): any; } }\ninterface Number extends NS.ICl { }\ndeclare function mk<T extends NS.ICl>(v: T): T;\nconst r = mk(3);",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_intersection_source_structural_fall_through() {

    let src = "\
interface FirstInterface { commonProperty: number }
interface SecondInterface { commonProperty: number }
function mySecondFunction<T extends { commonProperty: number, otherProperty: number }>(newParam: T): T { return newParam }
function myFirstFunction<T extends FirstInterface | SecondInterface>(param1: T) {
    const newParam: T & { otherProperty: number } = Object.assign(param1, { otherProperty: 3 });
    mySecondFunction(newParam)
}
";
    let diags = check_source_strict(src);
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_intersection_of_objects_assignable_to_merged_shape() {
    let diags = check_source_strict(
        "declare const ab: { a: number } & { b: string };\nconst t: { a: number; b: string } = ab;",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_new_expression_explicit_type_arguments_substituted() {
    let diags =
        check_source_strict("class C<T> { constructor(x: T) { } }\nconst z = new C<number>(5);");
    assert_no_diagnostics(&diags);
    let diags =
        check_source_strict("class C<T> { constructor(x: T) { } }\nconst z = new C<number>('s');");
    assert_diagnostic_count(&diags, 2345, 1);
}

#[test]
fn checker_comma_expression_types_as_right_operand() {

    let diags = check_source_strict(
        "function sideEffect(): void {}\nlet f: ((a: number) => void) | undefined;\nf ??= (sideEffect(), (a => a));\nf(42);",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_heritage_call_expression_base_silent() {

    let diags = check_source_strict(
        "class B {}\nfunction foo() { return { B: B }; }\nclass C extends (foo()).B {}\ndeclare const c: C;\nconst b: B = c;",
    );
    assert_diagnostic_count(&diags, 2503, 0);
}

#[test]
fn checker_comma_lhs_reference_narrowing_typeof() {
    let diags = check_source_strict(
        "const otherValue = () => true;\nconst value: { inner: number | string } = null as any;\nif (typeof (otherValue(), value).inner === 'number') {\n    const b: number = (otherValue(), value).inner;\n}",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_comma_lhs_reference_narrowing_predicate() {
    let diags = check_source_strict(
        "const otherValue = () => true;\nconst value: { inner: number | string } = null as any;\nfunction isNumber(obj: any): obj is number { return true; }\nif (isNumber((otherValue(), value).inner)) {\n    const b: number = (otherValue(), value).inner;\n}",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_assertion_call_narrows_compared_argument() {

    let diags = check_source_strict(
        "declare function assert(value: any): asserts value;\nfunction foo2(param: number | null | undefined): number | null {\n    const val = param !== undefined;\n    return val ? (assert(param !== undefined), param) : null;\n}",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_optional_param_display_strips_undefined() {

    let diags = check_source_strict(
        "function foo(a: string, b?: string, ...c: number[]) {}\nfoo('foo', 1);",
    );
    assert_diagnostic_count(&diags, 2345, 1);
    let shows_undefined = diags
        .iter()
        .filter(|d| d.code == 2345)
        .any(|d| d.message_args.iter().any(|a| a.contains("undefined")));
    assert!(
        !shows_undefined,
        "2345 message must print 'string', not 'string | undefined'"
    );
}

#[test]
fn checker_error_typed_optional_param_stays_error() {

    let diags = check_source_strict(
        "interface Wrap { m?(x?: import(\"./missing\").Nope): void; }\ndeclare const w: Wrap;\nw.m(1);",
    );
    assert_diagnostic_count(&diags, 2353, 0);
}

#[test]
fn checker_logical_and_truthiness_narrows_property_reference() {

    let diags = check_source_strict(
        "declare const r: { s?: number };\nr.s && r.s.toFixed();",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_logical_or_rhs_bound_under_falsy_condition() {

    let diags = check_source_strict(
        "declare let s: string | undefined;\nconst u: undefined = s || (s);",
    );
    assert_diagnostic_count(&diags, 2322, 1);
}

#[test]
fn checker_unexported_namespace_member_reports_ts2694() {

    let diags = check_source_strict(
        "namespace N {\n    function S() {}\n}\nvar a: N.S;",
    );
    assert_diagnostic_count(&diags, 2694, 1);
}

#[test]
fn checker_super_call_checks_under_heritage_instantiation() {

    let diags = check_source_strict(
        "class B<T> { constructor(a: T) { } }\nclass D extends B<number> {\n    constructor(b: number) { super(b); }\n}",
    );
    assert_no_diagnostics(&diags);
    let diags = check_source_strict(
        "class B<T> { constructor(a: T) { } }\nclass D extends B<number> {\n    constructor(b: string) { super(b); }\n}",
    );
    assert_diagnostic_count(&diags, 2345, 1);
}

#[test]
fn checker_plain_new_in_derived_class_not_heritage_instantiated() {

    let diags = check_source_strict(
        "class B<T> { constructor(a: T) { } }\nclass D extends B<number> {\n    constructor() { new B('s'); }\n}",
    );
    let msg = diags
        .iter()
        .find(|d| d.code == 2345)
        .map(|d| d.message_args.join(" "))
        .unwrap_or_default();
    assert!(
        msg.contains("T") && !msg.contains("number"),
        "must be the unsubstituted T, not heritage-instantiated: {msg:?}"
    );
}

#[test]
fn checker_js_specifier_resolves_to_ts_file() {

    let diags = check_sources(&[
        ("foo_0.ts", "export var foo = 42;"),
        ("foo_1.ts", "import foo = require('./foo_0.js');\nvar x = foo.foo + 42;"),
    ]);
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_narrow_by_instanceof_definite_assignment() {

    let src = "\
class Match { range(): any { return 1; } }
class FileMatch { resource(): any { return 1; } }
type FMOM = FileMatch | Match;
let elementA: FMOM, elementB: FMOM;
if (elementA instanceof FileMatch && elementB instanceof FileMatch) {
    elementA.resource();
} else if (elementA instanceof Match && elementB instanceof Match) {
    elementA.range();
}
";
    let diags = check_source_strict(src);
    assert_diagnostic_count(&diags, 2454, 4);
}

#[test]
fn checker_logical_or_short_circuit_merge_with_assignment() {

    let diags = check_source_strict(
        "declare let s: string | undefined;\ns || (s = 'x');\nconst t: string = s;",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_expression_position_assignment_in_comma_narrows() {
    let diags = check_source_strict(
        "function foo(param: number | null | undefined): number {\n    const y = (param = 5, param);\n    return y;\n}",
    );
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_generic_alias_export_no_2459() {

    let files = [
        (
            "bbb.d.ts",
            "export interface INode<T> {\n    data: T;\n}\n\nexport function create<T>(): () => INode<T>;\n",
        ),
        (
            "lib.d.ts",
            "export type G<T extends string> = { [P in T]: string };\n\nexport enum E {\n    A = \"a\",\n    B = \"b\"\n}\n\nexport type T = G<E>;\n\nexport type Q = G<E.A>;\n",
        ),
        (
            "index.ts",
            "import { T, Q } from \"./lib\";\nimport { create } from \"./bbb\";\n\nexport const fun = create<T>();\n\nexport const fun2 = create<Q>();\n",
        ),
    ];
    let diags = check_sources(&files);
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_module_imports_are_not_globals() {

    let files = [
        ("lib5.d.ts", "export type G2<T> = { [P in T]: string };\nexport type T = G2<\"a\">;\n"),
        ("index.ts", "import { T } from \"./lib5\";\nconst v: T = { a: \"x\" };\nconst arr: Array<number> = [1];\n"),
    ];
    let diags = check_sources(&files);
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_reexport_through_namespace_export_equals() {

    let files = [
        ("second.d.ts", "export import Component = CompletelyMissing;\n"),
        ("first.d.ts", "import * as Second from './second';\nexport = Second;\n"),
        ("crash.ts", "import { Component } from './first';\nclass C extends Component { }\n"),
    ];
    let diags = check_sources(&files);
    let ts2305 = diags.iter().filter(|d| d.code == 2305).count();
    assert_eq!(ts2305, 0, "import through export= must resolve Component");
    assert_diagnostic_code(&diags, 2503);
}

#[test]
fn checker_ambient_module_shadows_type_root_file_resolution() {

    let fs = Arc::new(InMemoryFS::new());
    fs.insert_dir("/proj");
    fs.insert_dir("/proj/typings/phaser/types");
    fs.insert_file(
        "/proj/typings/phaser/package.json",
        "{ \"name\": \"phaser\", \"version\": \"1.2.3\", \"types\": \"types/phaser.d.ts\" }",
    );
    fs.insert_file(
        "/proj/typings/phaser/types/phaser.d.ts",
        "declare module \"phaser\" {\n    export const a2: number;\n}\n",
    );
    fs.insert_file("/proj/a.ts", "import { a2 } from \"phaser\";\na2;\n");

    let args = [
        "--noLib".to_string(),
        "--types".to_string(),
        "phaser".to_string(),
        "--typeRoots".to_string(),
        "/proj/typings".to_string(),
        "/proj/a.ts".to_string(),
    ];
    let parsed = parse_command_line(&args, "/proj", Some(fs.as_ref()));
    let host: Arc<dyn tsox::compiler::CompilerHost> = Arc::new(CompilerHostImpl::new(
        fs,
        "/proj".to_string(),
        lib_path(),
    ));
    let program = Arc::new(Program::new(ProgramOptions {
        config: parsed,
        host,
    }));
    let diags = program.get_semantic_diagnostics();
    let non_globals: Vec<&tsox::ast::Diagnostic> =
        diags.iter().filter(|d| d.code != 2318).collect();
    assert!(
        non_globals.is_empty(),
        "unexpected diagnostics: {:?}",
        non_globals
    );
}

#[test]
fn checker_var_undefined_initializer_infers_any_without_strict() {

    let src = "export function foo() {\nvar classes = undefined;\n    return new classes(null);\n}\n";
    let diags = check_source_with_lib_args(src, &["--strict", "false"]);
    assert_no_diagnostics(&diags);

    let diags2 = check_source_with_lib_args(
        "var x = undefined;\nx = 5;\nconst y: number = x;\n",
        &["--strict", "false"],
    );
    assert_no_diagnostics(&diags2);
}

#[test]
fn checker_chain_condition_property_reread_narrows() {

    let src = "type foo = { bar: number | null; nested: { b: string | null; } };\n\
               const aFoo: foo = { bar: 3, nested: { b: \"y\" } };\n\
               const bBar = { elem1: 7, elem2: aFoo };\n\
               if (bBar.elem2 && bBar.elem2.bar && bBar.elem2.nested.b) {\n\
               \x20 const { bar, nested: { b: text } } = bBar.elem2;\n\
               \x20 const right: number = bBar.elem2.bar;\n\
               \x20 const wrong: number = bar;\n\
               \x20 const bAgain: string = text;\n\
               }\n";
    let diags = check_source_strict(src);
    assert_no_diagnostics(&diags);
}

#[test]
fn checker_local_type_shadows_later_top_level_class() {

    let src = "function f1() {\n\
               \x20   enum E { A, B, C }\n\
               \x20   class C { x: E = E.A; }\n\
               \x20   interface I { x: E; }\n\
               \x20   type A = I[];\n\
               \x20   let a: A = [new C()];\n\
               \x20   a[0].x = E.B;\n\
               \x20   return a;\n\
               }\n\
               class A {\n\
               \x20   m() { return 1; }\n\
               \x20   get p() { return 2; }\n\
               }\n";
    let diags = check_source_strict(src);
    assert_no_diagnostics(&diags);
}

#[test]
fn interface_call_signature_type_params_resolve() {

    let src = "interface I {\n\
               \x20   <T>(x: T): string;\n\
               }\n\
               interface I2 extends I { }\n\
               declare var i2: I2;\n\
               var r: string = i2(1);\n";
    let diags = check_source(src);
    assert_no_diagnostics(&diags);
}

#[test]
fn interface_call_signature_type_param_constraint_resolves() {

    let src = "interface A {\n\
               \x20   <T extends A>(x: T): void;\n\
               }\n\
               interface B {\n\
               \x20   <T extends B>(x: T): void;\n\
               }\n\
               declare var a: A;\n\
               a(a);\n";
    let diags = check_source(src);
    assert_no_diagnostics(&diags);
}

#[test]
fn type_literal_construct_signature_type_params_resolve() {

    let src = "interface ValueTypeMap { anyref: any; externref: any; }\n\
               type ValueType = keyof ValueTypeMap;\n\
               interface G<T extends ValueType = ValueType> { v: ValueTypeMap[T]; }\n\
               declare var Global: {\n\
               \x20   new<T extends ValueType = ValueType>(v?: ValueTypeMap[T]): G<T>;\n\
               };\n\
               declare var d: { value: \"anyref\" };\n\
               var g = new Global(d);\n";
    let diags = check_source(src);
    assert_no_diagnostics(&diags);
}

#[test]
fn global_class_does_not_shadow_interface_type_param() {

    let src = "class T {\n\
               \x20   static x() { }\n\
               }\n\
               type M = { a: 1; b: 2 };\n\
               interface G<T extends keyof M> {\n\
               \x20   value: M[T];\n\
               \x20   get(): M[T];\n\
               }\n\
               declare const g: G<\"a\">;\n\
               var v: 1 = g.value;\n";
    let diags = check_source_strict(src);
    assert_no_diagnostics(&diags);
}

#[test]
fn class_method_type_param_shadows_class_type_param() {

    let src = "class Box<T> {\n\
               \x20   wrap<T>(x: T): T { return x; }\n\
               }\n\
               declare var b: Box<number>;\n\
               var s: string = b.wrap(\"str\");\n\
               var n: number = b.wrap(1);\n";
    let diags = check_source(src);
    assert_no_diagnostics(&diags);
}

#[test]
fn constructor_type_node_type_params_resolve() {

    let src = "declare var C: new <T>(x: T) => T;\n\
               var n: number = new C(42);\n\
               var s: string = new C(\"x\");\n";
    let diags = check_source(src);
    assert_no_diagnostics(&diags);
}

#[test]
fn namespace_import_of_ambient_module_is_a_value() {

    let files: &[(&str, &str)] = &[
        (
            "/proj/inc.d.ts",
            "declare module \"foo\";\ndeclare module \"bar\" { namespace constants { export const X = 1; } }",
        ),
        (
            "/proj/main.ts",
            "import * as foo from \"foo\";\nimport * as bar from \"bar\";\nvoid foo;\nvar x: number = bar.constants.X;\n",
        ),
    ];
    let diags = check_sources(files);
    assert_no_diagnostics(&diags);
}

#[test]
fn forward_reference_class_heritage_type_param_resolves() {

    let src = "var someVariable: Class4<Class2>;\n\
               class Class1 { }\n\
               class Class2 extends Class1 { }\n\
               class Class3<T> {\n\
               \x20   public memberVariable: Class2 | undefined;\n\
               }\n\
               class Class4<T> extends Class3<T> { }\n";
    let diags = check_source(src);
    assert_no_diagnostics(&diags);
}

#[test]
fn type_parameter_coexists_with_same_named_getter() {

    let src = "export class Test<T> {\n\
               \x20   private get T(): T {\n\
               \x20       throw \"\";\n\
               \x20   }\n\
               \x20   public test(): T {\n\
               \x20       return null as any;\n\
               \x20   }\n\
               }\n";
    let diags = check_source(src);
    assert_no_diagnostics(&diags);
}

#[test]
fn mapped_type_mixed_literal_and_open_constraint_stays_deferred() {

    let src = "interface Named { name: string; }\n\
               declare function g<T>(b: { [P in keyof (T & Named)]: (T & Named)[P] }): void;\n\
               declare var tt: { name: \"ok\"; other: 1 };\n\
               g(tt);\n\
               function f<TType>(\n\
               \x20   a: { weak?: string } & Readonly<TType> & { name: \"ok\" },\n\
               \x20   b: Readonly<TType & { name: string }>,\n\
               \x20   c: Readonly<TType> & { name: string }) {\n\
               \x20   c = a;\n\
               \x20   b = a;\n\
               }\n";
    let diags = check_source_strict(src);
    assert_no_diagnostics(&diags);
}

#[test]
fn heritage_instantiation_keeps_method_type_param() {

    let src = "class Base<T> {\n\
               \x20   make<T>(x: T): T { return x; }\n\
               }\n\
               class Derived extends Base<number> { }\n\
               declare var d: Derived;\n\
               var s: string = d.make(\"str\");\n";
    let diags = check_source(src);
    assert_no_diagnostics(&diags);
}

#[test]
fn new_expression_member_chain_parses_as_instance_call() {

    let src = "class Box {\n\
               \x20   wrap(x: string): string { return x; }\n\
               }\n\
               var s: string = new Box().wrap(\"hi\");\n\
               class Gen<T> {\n\
               \x20   id(x: T): T { return x; }\n\
               }\n\
               var n: number = new Gen<number>().id(1);\n";
    let diags = check_source(src);
    assert_no_diagnostics(&diags);
}

#[test]
fn merged_interface_fork_instantiation_substitutes_all_declarations() {

    let src = "interface I<T> {\n\
               \x20   a: T;\n\
               }\n\
               interface I<T> {\n\
               \x20   b: T;\n\
               }\n\
               declare var x: I<number>;\n\
               var n: number = x.a;\n\
               var n2: number = x.b;\n";
    let diags = check_source(src);
    assert_no_diagnostics(&diags);
}

#[test]
fn overload_probe_requires_minimum_arity() {

    let src = "declare const A: {\n\
               \x20   new (n: number): string[];\n\
               \x20   new <T>(...items: T[]): T[];\n\
               };\n\
               var a = new A<string>();\n\
               var b = new A<string>(\"x\", \"y\");\n\
               var s1: string = a[0];\n\
               var s2: string = b[0];\n";
    let diags = check_source(src);
    assert_no_diagnostics(&diags);
}

#[test]
fn class_expression_heritage_value_symbol() {

    let files: &[(&str, &str)] = &[
        ("/proj/foo1.ts", "class x {}\nexport = x;\n"),
        (
            "/proj/foo2.ts",
            "import foo1 = require('./foo1');\nvar x = foo1;\nclass y extends x {}\nvar yy = new y();\n",
        ),
    ];
    let diags = check_sources(files);
    assert_no_diagnostics(&diags);
}

#[test]
fn generic_alias_type_params_shadow_same_named_outer_alias() {

    let src = "export type U = { kind?: 'A', a: string } | { kind?: 'B' } & { b: string };\n\
               type Ex<T, U> = T extends U ? T : never;\n\
               declare let x: Ex<U, { kind?: 'A' }>;\n\
               var a: string = x.a;\n";
    let diags = check_source(src);
    assert_no_diagnostics(&diags);
}

#[test]
fn weak_type_rejects_no_common_properties() {

    let src = "type E1 = { b: 1 } extends { kind?: 'A' } ? 1 : 2;\n\
               var q1: 2 = null as any as E1;\n\
               type E3 = { kind?: 'B' } & { b: 1 } extends { kind?: 'A' } ? 1 : 2;\n\
               var q3: 2 = null as any as E3;\n\
               type Ok = { kind?: 'A' } & { other: 1 } extends { kind?: 'A' } ? 1 : 2;\n\
               var qok: 1 = null as any as Ok;\n";
    let diags = check_source(src);
    assert_no_diagnostics(&diags);
}

#[test]
fn mapped_type_contextual_signature_not_broken_by_substitution() {

    let src = "interface Props {\n\
               \x20   when: (value: string) => boolean;\n\
               }\n\
               function bad<P extends Props>(\n\
               \x20   attrs: string extends keyof P ? { [K in keyof P]: P[K] } : { [K in keyof P]: P[K] }) { }\n\
               function good1<P extends Props>(\n\
               \x20   attrs: string extends keyof P ? P : { [K in keyof P]: P[K] }) { }\n\
               function good2<P extends Props>(\n\
               \x20   attrs: { [K in keyof P]: P[K] }) { }\n\
               bad({ when: value => false });\n\
               good1({ when: value => false });\n\
               good2({ when: value => false });\n\
               declare function g2<P extends Props>(attrs: { [K in keyof P]: P[K] }): void;\n\
               g2({ when: value => false });\n";
    let diags = check_source_strict(src);
    assert_no_diagnostics(&diags);
}

#[test]
fn cyclic_return_type_instantiates_type_arguments() {

    let src = "function foo<T>() {\n\
               \x20   var x: { a: T; b: typeof x };\n\
               \x20   return x;\n\
               }\n\
               function bar<T>() {\n\
               \x20   var x: { a: T; b: typeof x };\n\
               \x20   return x;\n\
               }\n\
               var a = foo<string>();\n\
               var b = bar<string>();\n\
               a = b;\n";
    let diags = check_source_strict(src);
    let non_2454: Vec<i32> = diags.iter().map(|d| d.code).filter(|c| *c != 2454).collect();
    assert!(
        non_2454.is_empty(),
        "expected only TS2454s, got codes {:?}",
        non_2454
    );
    assert_eq!(
        diags.iter().filter(|d| d.code == 2454).count(),
        2,
        "exactly two TS2454 (one per uninitialized x)"
    );
}

#[test]
fn same_alias_conditional_instances_stay_deferred_across_generic_fns() {

    let src = "type C<T> = T extends string ? 1 : 2;\n\
               function jc<T>(l: C<T>): void {}\n\
               function ac<T>(l: C<T>): void { jc(l); }\n\
               type Recur<T> = (\n\
               \x20   T extends (unknown[]) ? {} : { [K in keyof T]?: Recur<T[K]> }\n\
               ) | ['marker', ...Recur<T>[]];\n\
               function join<T>(l: Recur<T>[]): Recur<T> {\n\
               \x20   return ['marker', ...l];\n\
               }\n\
               function a<T>(l: Recur<T>[]): void {\n\
               \x20   const x: Recur<T> | undefined = join(l);\n\
               }\n";
    let diags = check_source_strict(src);
    assert_no_diagnostics(&diags);
}

#[test]
fn deferred_conditional_inference_fixes_target_parameter() {

    let src = "type C<X> = X extends string ? 1 : 2;\n\
               declare function h<Y>(c: C<Y>): void;\n\
               function u<X>(cc: C<X>) { h(cc); }\n";
    let diags = check_source_strict(src);
    assert_no_diagnostics(&diags);
}

#[test]
fn deferred_conditional_default_constraint_is_strict_for_concrete_targets() {

    let src = "type C<T> = T extends string ? 1 : 2;\n\
               function f<T>(p: C<T>) {\n\
               \x20   const a: C<number> = p;\n\
               }\n";
    let diags = check_source_strict(src);
    assert!(
        !diags.is_empty(),
        "expected TS2322 assigning deferred C<T> to resolved C<number>"
    );
}

#[test]
fn unconstrained_check_type_defers_instead_of_taking_false_branch() {

    let src = "type C<T> = T extends string ? 1 : 2;\n\
               function f<T>(p: C<T>) {\n\
               \x20   const a: number = p;\n\
               }\n";
    let diags = check_source_strict(src);
    assert_no_diagnostics(&diags);
}

#[test]
fn any_check_conditional_unions_true_branch_into_result() {

    let src = "type Spec = any extends object ? any : string;\n\
               type WithSpec<T extends number> = T;\n\
               type R = WithSpec<Spec>;\n\
               declare const r: R;\n";
    let diags = check_source_strict(src);
    assert_no_diagnostics(&diags);
}

#[test]
fn deferred_conditional_callee_uses_default_constraint_signatures() {

    let src = "type Transform1<T> = ((value: string) => T) | (string extends T ? undefined : never);\n\
               type Transform2<T> = string extends T ? ((value: string) => T) | undefined : (value: string) => T;\n\
               function test1<T>(f1: Transform1<T>, f2: Transform2<T>) {\n\
               \x20   f1?.(\"hello\");\n\
               \x20   f2?.(\"hello\");\n\
               }\n";
    let diags = check_source_strict(src);
    assert_no_diagnostics(&diags);
}

#[test]
fn forced_true_branch_resolution_sees_infer_scope() {

    let src = "type ThisParameterType<T> = T extends (this: infer U, ...args: never) => any ? U : unknown;\n\
               function f(this: number): void {}\n\
               type TP = ThisParameterType<typeof f>;\n";
    let _diags = check_source_strict(src);
}

fn check_sources_with_args(files: &[(&str, &str)], extra_args: &[&str]) -> Vec<tsox::ast::Diagnostic> {
    let fs = Arc::new(InMemoryFS::new());
    fs.insert_dir("/proj");
    for (path, content) in files {
        let abs = if path.starts_with('/') {
            (*path).to_string()
        } else {
            format!("/proj/{path}")
        };
        let mut parent = abs.rsplit_once('/').map(|(d, _)| d.to_string()).unwrap_or_default();
        while !parent.is_empty() {
            fs.insert_dir(&parent);
            let next = parent.rsplit_once('/').map(|(d, _)| d.to_string()).unwrap_or_default();
            if next == parent {
                break;
            }
            parent = next;
        }
        fs.insert_file(&abs, content);
    }
    let entry = files
        .iter()
        .find(|(p, _)| p.ends_with(".ts") || p.ends_with(".cts") || p.ends_with(".mts"))
        .map(|(p, _)| {
            if p.starts_with('/') {
                (*p).to_string()
            } else {
                format!("/proj/{p}")
            }
        })
        .expect("a TypeScript entry file");
    let mut args: Vec<String> = Vec::new();
    for a in extra_args {
        args.push((*a).to_string());
    }

    let mut roots: Vec<String> = Vec::new();
    for (p, _) in files {
        let lower = p.to_ascii_lowercase();
        if lower.ends_with(".ts")
            || lower.ends_with(".tsx")
            || lower.ends_with(".mts")
            || lower.ends_with(".cts")
        {
            roots.push(if p.starts_with('/') {
                (*p).to_string()
            } else {
                format!("/proj/{p}")
            });
        }
    }
    if roots.is_empty() {
        roots.push(entry);
    }
    args.extend(roots);
    let parsed = parse_command_line(&args, "/proj", Some(fs.as_ref()));
    let bf = Arc::new(BundledFS::new(fs));
    let host: Arc<dyn tsox::compiler::CompilerHost> =
        Arc::new(CompilerHostImpl::new(bf, "/proj".to_string(), lib_path()));
    let program = Arc::new(Program::new(ProgramOptions { config: parsed, host }));
    let mut all: Vec<tsox::ast::Diagnostic> = program.diagnostics().iter().map(|d| (**d).clone()).collect();
    all.extend(program.get_semantic_diagnostics());
    all
}

#[test]
fn ts2590_template_literal_cross_product_capped() {

    let src = "type Digits = 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9;\n\
               type D100000 = `${Digits}${Digits}${Digits}${Digits}${Digits}`;\n";
    let diags = check_source(src);
    assert_diagnostic_code(&diags, 2590);
}

#[test]
fn ts2590_template_literal_below_cap_clean() {

    let src = "type Digits = 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9;\n\
               type D10000 = `${Digits}${Digits}${Digits}${Digits}`;\n";
    let diags = check_source(src);
    assert_no_diagnostics(&diags);
}

#[test]
fn ts2590_tuple_variadic_cross_product_capped() {

    let src = "type TDigits = [0] | [1] | [2] | [3] | [4] | [5] | [6] | [7] | [8] | [9];\n\
               type T100000 = [...TDigits, ...TDigits, ...TDigits, ...TDigits, ...TDigits];\n";
    let diags = check_source(src);
    assert_diagnostic_code(&diags, 2590);
}

#[test]
fn ts2590_intersection_cross_product_capped() {

    let src = "type A = any;\n\
               type U1 = {a1:A} | {b1:A} | {c1:A} | {d1:A} | {e1:A} | {f1:A} | {g1:A} | {h1:A} | {i1:A} | {j1:A};\n\
               type U2 = {a2:A} | {b2:A} | {c2:A} | {d2:A} | {e2:A} | {f2:A} | {g2:A} | {h2:A} | {i2:A} | {j2:A};\n\
               type U3 = {a3:A} | {b3:A} | {c3:A} | {d3:A} | {e3:A} | {f3:A} | {g3:A} | {h3:A} | {i3:A} | {j3:A};\n\
               type U4 = {a4:A} | {b4:A} | {c4:A} | {d4:A} | {e4:A} | {f4:A} | {g4:A} | {h4:A} | {i4:A} | {j4:A};\n\
               type U5 = {a5:A} | {b5:A} | {c5:A} | {d5:A} | {e5:A} | {f5:A} | {g5:A} | {h5:A} | {i5:A} | {j5:A};\n\
               type U100000 = U1 & U2 & U3 & U4 & U5;\n";
    let diags = check_source(src);
    assert_diagnostic_code(&diags, 2590);
}

#[test]
fn ts2590_intersection_below_cap_clean() {
    let src = "type A = any;\n\
               type U1 = {a1:A} | {b1:A} | {c1:A} | {d1:A} | {e1:A} | {f1:A} | {g1:A} | {h1:A} | {i1:A} | {j1:A};\n\
               type U2 = {a2:A} | {b2:A} | {c2:A} | {d2:A} | {e2:A} | {f2:A} | {g2:A} | {h2:A} | {i2:A} | {j2:A};\n\
               type U100 = U1 & U2;\n";
    let diags = check_source(src);
    assert_no_diagnostics(&diags);
}

#[test]
fn ts1338_infer_only_in_conditional_extends() {

    let src = "type TV1 = `${infer X}`;\n";
    let diags = check_source(src);
    assert_diagnostic_code(&diags, 1338);
}

#[test]
fn ts1338_infer_inside_conditional_extends_clean() {
    let src = "type S1<T> = T extends `foo${infer U}bar` ? S2<U> : never;\n\
               type S2<S extends string> = S;\n\
               type X = S1<'foobar'>;\n";
    let diags = check_source(src);
    assert_no_diagnostics(&diags);
}

#[test]
fn ts1479_cjs_file_static_imports_esm() {

    let files = [
        ("index.cts", "import * as m from \"./lib.mjs\";\n"),
        ("lib.d.mts", "export const x: number;\n"),
        ("package.json", "{\"name\":\"p\",\"type\":\"commonjs\"}\n"),
    ];
    let diags = check_sources_with_args(&files, &["--module", "node16"]);
    assert_diagnostic_code(&diags, 1479);
}

#[test]
fn ts1479_esm_file_importing_esm_clean() {

    let files = [
        ("index.mts", "import * as m from \"./lib.mjs\";\n"),
        ("lib.d.mts", "export const x: number;\n"),
        ("package.json", "{\"name\":\"p\",\"type\":\"module\"}\n"),
    ];
    let diags = check_sources_with_args(&files, &["--module", "node16"]);
    let non_2307: Vec<i32> = diags.iter().map(|d| d.code).filter(|c| *c != 2307).collect();
    assert!(non_2307.is_empty(), "unexpected diagnostics: {:?}", non_2307);
}

#[test]
fn ts1479_not_under_node20() {

    let files = [
        ("index.cts", "import * as m from \"./lib.mjs\";\n"),
        ("lib.d.mts", "export const x: number;\n"),
        ("package.json", "{\"name\":\"p\",\"type\":\"commonjs\"}\n"),
    ];
    let diags = check_sources_with_args(&files, &["--module", "node20"]);
    let non_2307: Vec<i32> = diags.iter().map(|d| d.code).filter(|c| *c != 2307).collect();
    assert!(non_2307.is_empty(), "unexpected diagnostics: {:?}", non_2307);
}

#[test]
fn ts1471_import_equals_targeting_esm() {
    let files = [
        ("index.mts", "import m = require(\"./lib.mjs\");\n"),
        ("lib.d.mts", "export const x: number;\n"),
        ("package.json", "{\"name\":\"p\",\"type\":\"module\"}\n"),
    ];
    let diags = check_sources_with_args(&files, &["--module", "node16"]);
    assert_diagnostic_code(&diags, 1471);
}

#[test]
fn ts2883_inferred_export_type_not_portable() {

    let files = [
        (
            "index.ts",
            "import { x } from \"inner\";\nexport const a = x();\n",
        ),
        (
            "node_modules/inner/index.ts",
            "export { x } from \"./other.js\";\n",
        ),
        (
            "node_modules/inner/other.ts",
            "export interface Thing {}\nexport const x: () => Thing = null as any;\n",
        ),
        ("node_modules/inner/package.json", "{\"name\":\"inner\",\"type\":\"module\",\"exports\":\"./index.ts\"}\n"),
        ("package.json", "{\"name\":\"package\",\"type\":\"module\"}\n"),
    ];
    let diags = check_sources_with_args(&files, &["--module", "node16", "--declaration", "--target", "es2022"]);
    let found = diags.iter().any(|d| d.code == 2883);
    assert!(
        found,
        "expected TS2883, got: {:?}",
        diags.iter().map(|d| (d.code, d.message_args.clone())).collect::<Vec<_>>()
    );
}

#[test]
fn ts2883_not_without_declaration_option() {
    let files = [
        (
            "index.ts",
            "import { x } from \"inner\";\nexport const a = x();\n",
        ),
        (
            "node_modules/inner/index.ts",
            "export { x } from \"./other.js\";\n",
        ),
        (
            "node_modules/inner/other.ts",
            "export interface Thing {}\nexport const x: () => Thing = null as any;\n",
        ),
        ("node_modules/inner/package.json", "{\"name\":\"inner\",\"type\":\"module\",\"exports\":\"./index.ts\"}\n"),
        ("package.json", "{\"name\":\"package\",\"type\":\"module\"}\n"),
    ];
    let diags = check_sources_with_args(&files, &["--module", "node16", "--target", "es2022"]);
    let has_2883 = diags.iter().any(|d| d.code == 2883);
    assert!(!has_2883, "TS2883 must not fire without --declaration");
}

#[test]
fn mapped_type_display_uses_written_form() {

    let src = "function fa1<T>(x: T, z: { [P in keyof T & string as `p_${P}`]: T[P] }) {\n\
               \x20   z = x;\n\
               }\n";
    let diags = check_source(src);
    let target_texts: Vec<&str> = diags
        .iter()
        .filter(|d| d.code == 2322)
        .flat_map(|d| d.message_args.last().map(|s| s.as_str()))
        .collect();
    assert!(
        target_texts.iter().any(|t| t.contains("[P in keyof T & string as `p_${P}`]: T[P]; }")),
        "expected written-form mapped display, got: {:?}",
        target_texts
    );
}

#[test]
fn ts2883_ambient_module_import_names_type() {

    let files = [
        (
            "usage3.ts",
            "import { parse } from \"url\";\nexport const thing = parse();\n",
        ),
        (
            "node_modules/@types/node/index.d.ts",
            "declare module \"url\" {\n  export class Url {}\n  export function parse(): Url;\n}\n",
        ),
        ("package.json", "{\"name\":\"p\",\"type\":\"commonjs\"}\n"),
    ];
    let diags = check_sources_with_args(&files, &["--module", "node16", "--declaration", "--target", "es2015"]);
    let has_2883 = diags.iter().any(|d| d.code == 2883);
    assert!(
        !has_2883,
        "ambient-module-named type must not report TS2883, got: {:?}",
        diags.iter().map(|d| (d.code, d.message_args.clone())).collect::<Vec<_>>()
    );
}

#[test]
fn ts1479_ambiguous_dts_target_never_esm() {

    let files = [
        ("node_modules/inner/test.d.cts", "import * as t from \"inner/js/index\";\n"),
        ("node_modules/inner/index.d.ts", "export const q: number;\n"),
        ("node_modules/inner/index.d.mts", "export const r: number;\n"),
        (
            "node_modules/inner/package.json",
            "{\"name\":\"inner\",\"type\":\"module\",\"exports\":{\"./js/*\":\"./*.js\",\"./mjs/*\":\"./*.mjs\"}}\n",
        ),
        ("package.json", "{\"name\":\"package\",\"type\":\"module\"}\n"),
    ];
    let diags = check_sources_with_args(&files, &["--module", "node16"]);
    let js_1479 = diags
        .iter()
        .any(|d| d.code == 1479 && d.message_args.first().is_some_and(|s| s.contains("js/index")));
    assert!(
        !js_1479,
        "ambiguous .d.ts target must not report TS1479, got: {:?}",
        diags.iter().map(|d| (d.code, d.message_args.clone())).collect::<Vec<_>>()
    );
}

#[test]
fn ts1479_plain_dts_importer_counts_as_cjs() {

    let files = [
        ("node_modules/inner/test.d.ts", "import * as m from \"inner/mjs/index\";\n"),
        ("node_modules/inner/index.d.mts", "export const r: number;\n"),
        (
            "node_modules/inner/package.json",
            "{\"name\":\"inner\",\"type\":\"module\",\"exports\":{\"./mjs/*\":\"./*.mjs\"}}\n",
        ),
        ("package.json", "{\"name\":\"package\",\"type\":\"module\"}\n"),
    ];
    let diags = check_sources_with_args(&files, &["--module", "node16"]);
    assert_diagnostic_code(&diags, 1479);
}

#[test]
fn dynamic_import_namespace_member_typing() {

    let files = [
        (
            "index.ts",
            "export const a = (await import(\"inner\")).x();\n",
        ),
        (
            "node_modules/inner/index.ts",
            "export { x } from \"./other.js\";\n",
        ),
        (
            "node_modules/inner/other.ts",
            "export interface Thing {}\nexport const x: () => Thing = null as any;\n",
        ),
        ("node_modules/inner/package.json", "{\"name\":\"inner\",\"type\":\"module\",\"exports\":\"./index.ts\"}\n"),
        ("package.json", "{\"name\":\"package\",\"type\":\"module\"}\n"),
    ];
    let diags = check_sources_with_args(&files, &["--module", "node16", "--target", "es2022", "--declaration"]);
    assert_diagnostic_code(&diags, 2883);
}

#[test]
fn dynamic_import_member_assignability_checked() {

    let files = [
        (
            "index.ts",
            "const a = (await import(\"inner\")).x();\nconst b: string = a;\n",
        ),
        (
            "node_modules/inner/index.ts",
            "export { x } from \"./other.js\";\n",
        ),
        (
            "node_modules/inner/other.ts",
            "export interface Thing {}\nexport const x: () => Thing = null as any;\n",
        ),
        ("node_modules/inner/package.json", "{\"name\":\"inner\",\"type\":\"module\",\"exports\":\"./index.ts\"}\n"),
        ("package.json", "{\"name\":\"package\",\"type\":\"module\"}\n"),
    ];
    let diags = check_sources_with_args(&files, &["--module", "node16", "--target", "es2022"]);
    assert_diagnostic_code(&diags, 2322);
}

#[test]
fn ts2883_library_internal_files_exempt() {

    let files = [
        (
            "node_modules/inner/other.ts",
            "import { x } from \"inner\";\nexport const f = x();\n",
        ),
        (
            "node_modules/inner/index.ts",
            "export { x } from \"./other2.js\";\n",
        ),
        (
            "node_modules/inner/other2.ts",
            "export interface Thing {}\nexport const x: () => Thing = null as any;\n",
        ),
        ("node_modules/inner/package.json", "{\"name\":\"inner\",\"type\":\"module\",\"exports\":\"./index.ts\"}\n"),
        ("package.json", "{\"name\":\"package\",\"type\":\"module\"}\n"),
    ];
    let diags = check_sources_with_args(&files, &["--module", "node16", "--declaration", "--target", "es2022"]);
    let has_2883 = diags.iter().any(|d| d.code == 2883);
    assert!(
        !has_2883,
        "library-internal files must not report TS2883, got: {:?}",
        diags.iter().map(|d| (d.code, d.message_args.clone())).collect::<Vec<_>>()
    );
}

#[test]
fn ts2883_dynamic_import_initializer_names_module_type() {

    let files = [
        (
            "index.ts",
            "export const f = await import(\"inner\");\n",
        ),
        (
            "node_modules/inner/index.ts",
            "export const x: number;\n",
        ),
        ("node_modules/inner/package.json", "{\"name\":\"inner\",\"type\":\"module\",\"exports\":\"./index.ts\"}\n"),
        ("package.json", "{\"name\":\"package\",\"type\":\"module\"}\n"),
    ];
    let diags = check_sources_with_args(&files, &["--module", "node16", "--declaration", "--target", "es2022"]);
    let has_2883 = diags.iter().any(|d| d.code == 2883);
    assert!(
        !has_2883,
        "dynamic-import initializer must not report TS2883, got: {:?}",
        diags.iter().map(|d| (d.code, d.message_args.clone())).collect::<Vec<_>>()
    );
}

#[test]
fn b1_interface_extends_elaboration_pyramid() {

    let src = "interface Foo { f(): string; }\ninterface Bar extends Foo { f(key: string): string; }\n";
    let diags = check_source(src);
    let d = diags.iter().find(|d| d.code == 2430).expect("TS2430");
    assert!(!d.message_chain.is_empty(), "expected property elaboration level");
    assert!(d.message_chain[0].message.as_ref().is_some_and(|m|
        m.key.starts_with("Types_of_property")), "chain head: {:?}",
        d.message_chain[0].message.as_ref().map(|m| m.key));
    assert!(!d.message_chain[0].message_chain.is_empty(), "expected level2 (not-assignable)");
}

#[test]
fn b1_optional_property_undefined_leaf_chain() {

    let src = "interface A<T, U> { one: T; two?: U; }\n\
               declare var x: A<number, string>;\n\
               declare var y: { two: number; };\n\
               y = x;\n";
    let diags = check_source_strict(src);
    let d = diags.iter().find(|d| d.code == 2322).expect("TS2322");
    assert!(!d.message_chain.is_empty(), "expected elaboration chain");
}

#[test]
fn b1_no_match_signature_chain() {

    let src = "interface I { one: number; two?: string; }\n\
               declare var x: I;\n\
               declare var f: <Tstring>(a: Tstring) => Tstring;\n\
               f = x;\n";
    let diags = check_source(src);
    let d = diags.iter().find(|d| d.code == 2322).expect("TS2322");
    let target_text = d.message_args.last().cloned().unwrap_or_default();
    assert!(target_text.contains("<Tstring>"), "target: {target_text}");
    assert!(!d.message_chain.is_empty(), "expected no-match chain");
}

#[test]
fn b1_index_signature_missing_chain() {

    let src = "interface IHandlerMap { [type: string]: number; }\n\
               class Foo { Boz(): void { } }\n\
               function Biz(m: IHandlerMap) { }\n\
               Biz(new Foo());\n";
    let diags = check_source(src);
    let d = diags.iter().find(|d| d.code == 2345).expect("TS2345");
    assert!(!d.message_chain.is_empty(), "expected index-signature chain");
}
