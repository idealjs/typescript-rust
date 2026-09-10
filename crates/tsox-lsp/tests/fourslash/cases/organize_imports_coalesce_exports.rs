use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: f.VerifyOrganizeImports("]
#[test]
fn organize_imports_coalesce_exports_sort_specifiers_case_insensitive() {
    let content = r#"export { default as M, a as n, B, y, Z as O } from "lib";
void 0;"#;
    let mut s = Session::new_for_test("organizeImports_coalesceExports_sortSpecifiersCaseInsensitive", content);
    // TODO: f.VerifyOrganizeImports(
}

#[ignore = "generator: f.VerifyOrganizeImports("]
#[test]
fn organize_imports_coalesce_exports_combine_namespace_re_exports() {
    let content = r#"export * from "lib";
export * from "lib";
void 0;"#;
    let mut s = Session::new_for_test("organizeImports_coalesceExports_combineNamespaceReExports", content);
    // TODO: f.VerifyOrganizeImports(
}

#[ignore = "generator: f.VerifyOrganizeImports("]
#[test]
fn organize_imports_coalesce_exports_combine_property_exports() {
    let content = r#"const x = 1, z = 2;
export { x };
export { z as y };
void 0;"#;
    let mut s = Session::new_for_test("organizeImports_coalesceExports_combinePropertyExports", content);
    // TODO: f.VerifyOrganizeImports(
}

#[ignore = "generator: f.VerifyOrganizeImports("]
#[test]
fn organize_imports_coalesce_exports_combine_property_re_exports() {
    let content = r#"export { x } from "lib";
export { y as z } from "lib";
void 0;"#;
    let mut s = Session::new_for_test("organizeImports_coalesceExports_combinePropertyReExports", content);
    // TODO: f.VerifyOrganizeImports(
}

#[ignore = "generator: // Namespace re-export and property re-export from same modu"]
#[test]
fn organize_imports_coalesce_exports_namespace_with_property_re_export() {
    // TODO: // Namespace re-export and property re-export from same module should not be combined.
    let content = r#"export * from "lib";
export { y } from "lib";
export { z } from "aaa";
void 0;"#;
    let mut s = Session::new_for_test("organizeImports_coalesceExports_namespaceWithPropertyReExport", content);
    // TODO: f.VerifyOrganizeImports(
}

#[ignore = "generator: f.VerifyOrganizeImports("]
#[test]
fn organize_imports_coalesce_exports_combine_many() {
    let content = r#"const x = 1, w = 2, z = 3, q = 4;
export { x };
export { w as y, z as default };
export { q as w };
void 0;"#;
    let mut s = Session::new_for_test("organizeImports_coalesceExports_combineMany", content);
    // TODO: f.VerifyOrganizeImports(
}

#[ignore = "generator: f.VerifyOrganizeImports("]
#[test]
fn organize_imports_coalesce_exports_combine_many_re_exports() {
    let content = r#"export { x as a, y } from "lib";
export * from "lib";
export { z as b } from "lib";
void 0;"#;
    let mut s = Session::new_for_test("organizeImports_coalesceExports_combineManyReExports", content);
    // TODO: f.VerifyOrganizeImports(
}

#[ignore = "generator: // Type-only exports should be kept separate from value expo"]
#[test]
fn organize_imports_coalesce_exports_keep_type_only_separate() {
    // TODO: // Type-only exports should be kept separate from value exports.
    let content = r#"const x = 1;
type y = string;
export { x };
export type { y };
export { z } from "aaa";
void 0;"#;
    let mut s = Session::new_for_test("organizeImports_coalesceExports_keepTypeOnlySeparate", content);
    // TODO: f.VerifyOrganizeImports(
}

#[ignore = "generator: f.VerifyOrganizeImports("]
#[test]
fn organize_imports_coalesce_exports_combine_type_only() {
    let content = r#"type x = string;
type y = number;
export type { x };
export type { y };
void 0;"#;
    let mut s = Session::new_for_test("organizeImports_coalesceExports_combineTypeOnly", content);
    // TODO: f.VerifyOrganizeImports(
}
