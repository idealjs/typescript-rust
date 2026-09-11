use tsox_lsp::fourslash::{self, Session};


#[test]
fn code_fix_require_in_ts3() {
    let content = r#"// @Filename: /a.ts
const { a, b: { c } } = [|require("a")|];"#;
    let mut s = Session::new_for_test("codeFixRequireInTs3", content);
    // TODO: f.VerifyCodeFixNotAvailable(t)
}
