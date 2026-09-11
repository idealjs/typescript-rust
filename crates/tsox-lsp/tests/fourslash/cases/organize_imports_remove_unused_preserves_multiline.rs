use tsox_lsp::fourslash::{self, Session};


#[test]
fn organize_imports_remove_unused_preserves_multiline() {
    let content = r#"import {
    a,
    b,
    c,
} from "module";

export { a, b, c };"#;
    let mut s = Session::new_for_test("organizeImports_removeUnused_preservesMultiline", content);
    // TODO: f.VerifyOrganizeImports(
}

#[test]
fn organize_imports_remove_unused_preserves_multiline_with_removal() {
    let content = r#"import {
    a,
    b,
    c,
} from "module";

export { a, c };"#;
    let mut s = Session::new_for_test("organizeImports_removeUnused_preservesMultilineWithRemoval", content);
    // TODO: f.VerifyOrganizeImports(
}

#[test]
fn organize_imports_remove_unused_uses_language_service_format_options() {
    let content = r#"import {
    a,
    b,
    c,
} from "module";

export { a, c };"#;
    let mut s = Session::new_for_test("organizeImports_removeUnusedUsesLanguageServiceFormatOptions", content);
    // TODO: preferences := lsutil.ParseUserPreferences(map[string]any{
    // TODO: f.VerifyOrganizeImports(
}
