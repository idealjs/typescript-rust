use tsox_lsp::fourslash::{self, Session};


#[test]
fn get_java_script_quick_info3() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @allowNonTsExtensions: true
// @Filename: Foo.js
/** @param {number[]} [a] */
function /**/f(a) { }"#;
    let mut s = Session::new_for_test("getJavaScriptQuickInfo3", content);
    fourslash::verify_quick_info_at(&mut s, "", "function f(a?: number[]): void", "");
}
