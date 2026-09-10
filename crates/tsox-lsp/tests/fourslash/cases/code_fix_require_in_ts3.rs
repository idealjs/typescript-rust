use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCodeFixNotAvailable"]
#[test]
fn code_fix_require_in_ts3() {
    let content = r#"// @Filename: /a.ts
const { a, b: { c } } = [|require("a")|];"#;
    let mut s = Session::new_for_test("codeFixRequireInTs3", content);
    fourslash::unsupported("VerifyCodeFixNotAvailable"); // f.VerifyCodeFixNotAvailable(t)
}
