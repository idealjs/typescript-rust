use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineHover"]
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
    let mut s = Session::new_for_test("quickInfoInheritDoc4", content);
    fourslash::unsupported("VerifyBaselineHover"); // f.VerifyBaselineHover(t)
}
