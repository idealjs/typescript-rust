use tsox_lsp::fourslash::{self, Session};


#[test]
fn quick_info_for_object_binding_element_name03() {
    let content = r#"interface Options {
    /**
     * A description of foo
     */
    foo: string;
}

function f({ foo }: Options) {
    foo/*1*/;
}"#;
    let mut s = Session::new_for_test("quickInfoForObjectBindingElementName03", content);
    // TODO: f.VerifyBaselineHover(t)
}
