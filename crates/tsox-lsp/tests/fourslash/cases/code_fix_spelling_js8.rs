use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyNoErrors"]
#[test]
fn code_fix_spelling_js8() {
    let content = r#"// @allowjs: true
// @noEmit: true
// @filename: a.js
var locals = {}
// @ts-expect-error
Object.keys(locale)"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyNoErrors"); // f.VerifyNoErrors(t)
}
