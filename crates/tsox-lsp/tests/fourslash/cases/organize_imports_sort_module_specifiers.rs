use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: f.VerifyOrganizeImports("]
#[test]
fn organize_imports_sort_module_specifiers_non_relative_vs_non_relative() {
    let content = r#"import x from "lib2";
import y from "lib1";
x; y;"#;
    let mut s = Session::new(content);
    // TODO: f.VerifyOrganizeImports(
}

#[ignore = "generator: f.VerifyOrganizeImports("]
#[test]
fn organize_imports_sort_module_specifiers_relative_vs_relative() {
    let content = r#"import x from "./lib2";
import y from "./lib1";
x; y;"#;
    let mut s = Session::new(content);
    // TODO: f.VerifyOrganizeImports(
}

#[ignore = "generator: f.VerifyOrganizeImports("]
#[test]
fn organize_imports_sort_module_specifiers_relative_vs_non_relative() {
    let content = r#"import x from "./lib";
import y from "lib";
x; y;"#;
    let mut s = Session::new(content);
    // TODO: f.VerifyOrganizeImports(
}

#[ignore = "generator: // Verify 'a' sorts before 'Z' (case-insensitive)"]
#[test]
fn organize_imports_sort_module_specifiers_case_insensitive() {
    // TODO: // Verify "a" sorts before "Z" (case-insensitive)
    let content = r#"import x from "Z";
import y from "a";
x; y;"#;
    let mut s = Session::new(content);
    // TODO: f.VerifyOrganizeImports(
}

#[ignore = "generator: // Verify 'A' sorts before 'z' (case-insensitive)"]
#[test]
fn organize_imports_sort_module_specifiers_case_insensitive_reverse() {
    // TODO: // Verify "A" sorts before "z" (case-insensitive)
    let content = r#"import x from "z";
import y from "A";
x; y;"#;
    let mut s = Session::new(content);
    // TODO: f.VerifyOrganizeImports(
}
