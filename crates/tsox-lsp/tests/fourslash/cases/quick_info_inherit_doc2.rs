use tsox_lsp::fourslash::{self, Session};


#[test]
fn quick_info_inherit_doc2() {
    let content = r#"// @noEmit: true
// @allowJs: true
// @Filename: quickInfoInheritDoc2.ts
class Base<T> {
    /**
     * Base.prop
     */
    prop: T | undefined;
}

class SubClass<T> extends Base<T> {
    /**
     * @inheritdoc
     * SubClass.prop
     */
    /*1*/prop: T | undefined;
}"#;
    let mut s = Session::new_for_test("quickInfoInheritDoc2", content);
    // TODO: f.VerifyBaselineHover(t)
}
