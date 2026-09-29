use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn quick_info_js_doc_function_new() {
    let content = r#"// @allowJs: true
// @Filename: Foo.js
/** @type {function (new: string, string): string} */
var f/**/;"#;
    let mut s = Session::new_for_test("quickInfoJSDocFunctionNew", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyQuickInfoIs(t, "var f: new (arg1: string) => string", "")
}
