use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_import_export_equals() {
    let content = r#"// @module: commonjs
// @esModuleInterop: false
// @allowSyntheticDefaultImports: false
// @Filename: /a.d.ts
declare function a(): void;
declare namespace a {
    export interface b {}
}
export = a;
// @Filename: /b.ts
a/*0*/;
let x: b/*1*/;"#;
    let mut s = Session::new_for_test("completionsImport_exportEquals", content);
    fourslash::go_to_marker(&mut s, "0");
    // TODO: f.VerifyCompletions(t, "0", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyApplyCodeActionFromCompletion(t, new("1"), &fourslash.ApplyCodeActionFromCompletionOptions{
    // TODO: f.VerifyApplyCodeActionFromCompletion(t, new("0"), &fourslash.ApplyCodeActionFromCompletionOptions{
}
