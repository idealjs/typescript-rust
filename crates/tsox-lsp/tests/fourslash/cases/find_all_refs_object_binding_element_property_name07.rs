use tsox_lsp::fourslash::{self, Session};


#[test]
fn find_all_refs_object_binding_element_property_name07() {
    let content = r#"let p, b;

p, [{ /*1*/a: p, b }] = [{ a: 10, b: true }];"#;
    let mut s = Session::new_for_test("findAllRefsObjectBindingElementPropertyName07", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1")
}
