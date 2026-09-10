use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCodeFixAll"]
#[test]
fn import_name_code_fix_all_js() {
    let content = r#"// @module: esnext
// @allowJs: true
// @checkJs: true
// @Filename: /a.js
export class C {}
/** @typedef {number} T */
// @Filename: /b.js
C;
/** @type {T} */
const x = 0;"#;
    let mut s = Session::new_for_test("importNameCodeFix_all_js", content);
    fourslash::go_to_file(&mut s, "/b.js");
    fourslash::unsupported("VerifyCodeFixAll"); // f.VerifyCodeFixAll(t, fourslash.VerifyCodeFixAllOptions{
}
