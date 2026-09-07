use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCodeFixAll"]
#[test]
fn import_name_code_fix_require_named_and_default() {
    let content = r#"// @allowJs: true
// @checkJs: true
// @Filename: blah.ts
export default class Blah {}
export const Named1 = 0;
export const Named2 = 1;
// @Filename: index.js
Named1 + Named2;
new Blah;"#;
    let mut s = Session::new(content);
    fourslash::go_to_file(&mut s, "index.js");
    fourslash::unsupported("VerifyCodeFixAll"); // f.VerifyCodeFixAll(t, fourslash.VerifyCodeFixAllOptions{
}
