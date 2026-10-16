use tsox_lsp::fourslash::Session;


#[test]
fn quick_info_inherit_doc4() {
    let content = r#"// @Filename: quickInfoInheritDoc4.ts
var A: any;

class B extends A {
    /**
     * @inheritdoc
     */
    static /**/value() {
        return undefined;
    }
}"#;
    let _s = Session::new_for_test("quickInfoInheritDoc4", content);
    // TODO: f.VerifyBaselineHover(t)
}
