use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineHover"]
#[test]
fn quick_info_for_object_binding_element_name05() {
    let content = r#"interface A {
    /**
     * A description of a
     */
    a: number;
}
interface B {
    a: string;
}

function f({ a }: A | B) {
    a/**/;
}"#;
    let mut s = Session::new_for_test("quickInfoForObjectBindingElementName05", content);
    fourslash::unsupported("VerifyBaselineHover"); // f.VerifyBaselineHover(t)
}
