use tsox_lsp::fourslash::{self, Session};


#[test]
fn find_all_refs_object_binding_element_property_name02() {
    let content = r#"interface I {
    /*1*/property1: number;
    property2: string;
}

var foo: I;
/*2*/var { /*3*/property1: {} } = foo;"#;
    let mut s = Session::new_for_test("findAllRefsObjectBindingElementPropertyName02", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2", "3")
}
