use tsox_lsp::fourslash::{self, Session};


#[test]
fn get_java_script_completions15() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @allowNonTsExtensions: true
// @Filename: refFile1.ts
export var V = 1;
// @Filename: refFile2.ts
export var V = "123"
// @Filename: refFile3.ts
export var V = "123"
// @Filename: main.js
import ref1 = require("./refFile1");
var ref2 = require("./refFile2");
ref1.V./*1*/;
ref2.V./*2*/;
var v = { x: require("./refFile3") };
v.x./*3*/;
v.x.V./*4*/;"#;
    let mut s = Session::new_for_test("getJavaScriptCompletions15", content);
    fourslash::verify_completions_include_exclude_at(&mut s, Some("1"), &["toExponential"], &[]);
    fourslash::verify_completions_include_exclude_at(&mut s, Some("2"), &["toLowerCase"], &[]);
    // TODO: f.VerifyCompletions(t, "3", &fourslash.CompletionsExpectedList{
    fourslash::verify_completions_include_exclude_at(&mut s, Some("4"), &["toLowerCase"], &[]);
}
