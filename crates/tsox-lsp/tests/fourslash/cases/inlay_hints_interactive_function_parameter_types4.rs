use tsox_lsp::fourslash::Session;


#[test]
fn inlay_hints_interactive_function_parameter_types4() {
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
    let _s = Session::new_for_test("inlayHintsInteractiveFunctionParameterTypes4", content);
    // TODO: f.VerifyBaselineInlayHints(t, nil /*span*/, &lsutil.UserPreferences{InlayHints: lsutil.InlayHintsPre
}
