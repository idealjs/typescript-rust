use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineInlayHints"]
#[test]
fn inlay_hints_function_parameter_types4() {
    let content = r#"// @allowJs: true
// @checkJs: true
// @Filename: /a.js
class Foo {
    #value = 0;
    get foo() { return this.#value; }
    /**
     * @param {number} value
     */
    set foo(value) { this.#value = value; }
}"#;
    let mut s = Session::new_for_test("inlayHintsFunctionParameterTypes4", content);
    fourslash::unsupported("VerifyBaselineInlayHints"); // f.VerifyBaselineInlayHints(t, nil /*span*/, &lsutil.UserPreferences{InlayHints: lsutil.InlayHintsPre
}
