use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineHover"]
#[test]
fn quick_info_for_object_binding_element_name06() {
    let content = r#"type Foo = {
    /**
     * Thing is a bar
     */
    isBar: boolean

    /**
     * Thing is a baz
     */
    isBaz: boolean
}

function f(): Foo {
    return undefined as any
}

const { isBaz: isBar } = f();
isBar/**/;"#;
    let mut s = Session::new_for_test("quickInfoForObjectBindingElementName06", content);
    fourslash::unsupported("VerifyBaselineHover"); // f.VerifyBaselineHover(t)
}
