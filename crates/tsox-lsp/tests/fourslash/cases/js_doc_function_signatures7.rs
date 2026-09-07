use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
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
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("VerifyQuickInfoIs"); // f.VerifyQuickInfoIs(t, "var test: Test", "")
}
