use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn quick_info_js_doc_function_this() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @allowJs: true
// @Filename: Foo.js
/** @type {function (this: string, string): string} */
var f/**/ = function (s) { return s; }"#;
    let mut s = Session::new_for_test("quickInfoJSDocFunctionThis", content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("VerifyQuickInfoIs"); // f.VerifyQuickInfoIs(t, "var f: (this: string, arg1: string) => string", "")
}
