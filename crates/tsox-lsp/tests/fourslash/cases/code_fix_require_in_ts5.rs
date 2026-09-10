use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCodeFixNotAvailable"]
#[test]
fn code_fix_require_in_ts5() {
    let content = r#"// @Filename: /a.ts
const a = 1;
const b = 2;
const foo = require(` + "`" + `foo${a}${b}` + "`" + `);"#;
    let mut s = Session::new_for_test("codeFixRequireInTs5", content);
    fourslash::unsupported("VerifyCodeFixNotAvailable"); // f.VerifyCodeFixNotAvailable(t)
}
