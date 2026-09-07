use tsox_lsp::fourslash::{self, Session};

#[test]
fn code_fix_spelling_js8() {
    let content = r#"// @allowjs: true
// @noEmit: true
// @filename: a.js
var locals = {}
// @ts-expect-error
Object.keys(locale)"#;
    let mut s = Session::new(content);
    fourslash::verify_no_errors(&mut s);
}
