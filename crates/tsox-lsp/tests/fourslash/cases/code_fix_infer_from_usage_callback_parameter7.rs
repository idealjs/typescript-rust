use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCodeFixNotAvailable"]
#[test]
fn code_fix_infer_from_usage_callback_parameter7() {
    let content = r#"// @allowJs: true
// @checkJs: true
// @noImplicitAny: false
// @filename: /foo.js
/** @type {(x: number) => number} */
const foo = x => x + 1;"#;
    let mut s = Session::new_for_test("codeFixInferFromUsageCallbackParameter7", content);
    fourslash::unsupported("VerifyCodeFixNotAvailable"); // f.VerifyCodeFixNotAvailable(t)
}
