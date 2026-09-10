use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completions_paths_import_type() {
    let content = r#"// @allowJs: true
// @moduleResolution: bundler
// @Filename: /ns.ts
file content not read
// @Filename: /node_modules/package/index.ts
file content not read
// @Filename: /usage.ts
type A = typeof import("[|p|]/*1*/");
type B = import(".//*2*/");
// @Filename: /user.js
/** @type {import("/*3*/")} */"#;
    let mut s = Session::new_for_test("completionsPaths_importType", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "3", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
}
