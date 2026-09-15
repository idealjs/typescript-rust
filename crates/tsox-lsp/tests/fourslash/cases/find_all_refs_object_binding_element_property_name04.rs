use tsox_lsp::fourslash::Session;


#[test]
fn find_all_refs_object_binding_element_property_name04() {
    let content = r#"interface I {
    /*0*/property1: number;
    property2: string;
}

function f({ /*1*/property1: p1 }: I,
           { /*2*/property1 }: I,
           { property1: p2 }) {

    return /*3*/property1 + 1;
}"#;
    let _s = Session::new_for_test("findAllRefsObjectBindingElementPropertyName04", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "0", "1", "2", "3")
}
