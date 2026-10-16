use tsox_lsp::fourslash::Session;


#[test]
fn find_all_refs_object_binding_element_property_name01() {
    let content = r#"interface I {
    /*1*/property1: number;
    property2: string;
}

var foo: I;
/*2*/var { /*3*/property1: prop1 } = foo;"#;
    let _s = Session::new_for_test("findAllRefsObjectBindingElementPropertyName01", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2", "3")
}
