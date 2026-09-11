use tsox_lsp::fourslash::{self, Session};


#[test]
fn js_doc_function_signatures7() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @allowJs: true
// @Filename: Foo.js
/**
 * @param {string} p0
 * @param {string} [p1]
 */
function Test(p0, p1) {
    this.P0 = p0;
    this.P1 = p1;
}


var /**/test = new Test("");"#;
    let mut s = Session::new_for_test("jsDocFunctionSignatures7", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyQuickInfoIs(t, "var test: Test", "")
}
