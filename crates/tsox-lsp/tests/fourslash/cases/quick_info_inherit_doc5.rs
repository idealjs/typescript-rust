use tsox_lsp::fourslash::Session;


#[test]
fn quick_info_inherit_doc5() {
    let content = r#"// @allowJs: true
// @checkJs: true
// @Filename: quickInfoInheritDoc5.js
function A() {}

class B extends A {
    /**
     * @inheritdoc
     */
    static /**/value() {
        return undefined;
    }
}"#;
    let _s = Session::new_for_test("quickInfoInheritDoc5", content);
    // TODO: f.VerifyBaselineHover(t)
}
