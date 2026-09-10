use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn triple_slash_ref_path_completion_extensions_allow_js_false() {
    let content = r#"// @Filename: test0.ts
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
    let mut s = Session::new_for_test("tripleSlashRefPathCompletionExtensionsAllowJSFalse", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, f.Markers(), &fourslash.CompletionsExpectedList{
}
