use tsox_lsp::fourslash::{self, Session};


#[test]
fn triple_slash_ref_path_completion_extensions_allow_js_true() {
    let content = r#"// @allowJs: true
// @Filename: test0.ts
/// <reference path="/*0*/
/// <reference path=".//*1*/
/// <reference path="./f/*2*/
// @Filename: f1.ts

// @Filename: f1.js

// @Filename: f1.d.ts

// @Filename: f1.tsx

// @Filename: f1.js

// @Filename: f1.jsx

// @Filename: f1.cs
"#;
    let mut s = Session::new_for_test("tripleSlashRefPathCompletionExtensionsAllowJSTrue", content);
    // TODO: f.VerifyCompletions(t, f.Markers(), &fourslash.CompletionsExpectedList{
}
