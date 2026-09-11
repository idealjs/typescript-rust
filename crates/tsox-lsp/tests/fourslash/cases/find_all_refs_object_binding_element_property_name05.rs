use tsox_lsp::fourslash::{self, Session};


#[test]
fn find_all_refs_object_binding_element_property_name05() {
    let content = r#"interface I {
    property1: number;
    property2: string;
}

function f({ /**/property1: p }, { property1 }) {
    let x = property1;
}"#;
    let mut s = Session::new_for_test("findAllRefsObjectBindingElementPropertyName05", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "")
}
