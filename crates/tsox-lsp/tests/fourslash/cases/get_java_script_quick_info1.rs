use tsox_lsp::fourslash::{self, Session};


#[test]
fn get_java_script_quick_info1() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @allowNonTsExtensions: true
// @Filename: Foo.js
/** @type {function(new:string,number)} */
var /**/v;"#;
    let mut s = Session::new_for_test("getJavaScriptQuickInfo1", content);
    fourslash::verify_quick_info_at(&mut s, "", "var v: new (arg1: number) => string", "");
}
