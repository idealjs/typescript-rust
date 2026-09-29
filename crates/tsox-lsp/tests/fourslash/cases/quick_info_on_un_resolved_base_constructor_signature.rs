use tsox_lsp::fourslash::{self, Session};


#[test]
fn quick_info_on_un_resolved_base_constructor_signature() {
    let content = r#"class baseClassWithConstructorParameterSpecifyingType {
    constructor(loading?: boolean) {
    }
}
class genericBaseClassInheritingConstructorFromBase<TValue> extends baseClassWithConstructorParameterSpecifyingType {
}
class classInheritingSpecializedClass extends genericBaseClassInheritingConstructorFromBase<string> {
}
new class/*1*/InheritingSpecializedClass();"#;
    let mut s = Session::new_for_test("quickInfoOnUnResolvedBaseConstructorSignature", content);
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.VerifyQuickInfoExists(t)
}
