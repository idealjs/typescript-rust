use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineHover"]
#[test]
fn quick_info_inherit_doc3() {
    let content = r#"// @noEmit: true
// @allowJs: true
// @Filename: quickInfoInheritDoc3.ts
function getBaseClass() {
    return class Base {
        /**
         * Base.prop
         */
        prop: string | undefined;
    }
}
class SubClass extends getBaseClass() {
    /**
     * @inheritdoc
     * SubClass.prop
     */
    /*1*/prop: string | undefined;
}"#;
    let mut s = Session::new_for_test("quickInfoInheritDoc3", content);
    fourslash::unsupported("VerifyBaselineHover"); // f.VerifyBaselineHover(t)
}
