use tsox_lsp::fourslash::{self, Session};


#[test]
fn get_java_script_quick_info6() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @allowNonTsExtensions: true
// @Filename: Foo.js
/** @type {function(this:number)} */
function f() { /**/this }"#;
    let mut s = Session::new_for_test("getJavaScriptQuickInfo6", content);
    fourslash::verify_quick_info_at(&mut s, "", "number", "");
}
