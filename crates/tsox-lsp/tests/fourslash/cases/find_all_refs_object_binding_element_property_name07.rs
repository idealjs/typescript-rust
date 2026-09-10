use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn find_all_refs_object_binding_element_property_name07() {
    let content = r#"let p, b;

p, [{ /*1*/a: p, b }] = [{ a: 10, b: true }];"#;
    let mut s = Session::new_for_test("findAllRefsObjectBindingElementPropertyName07", content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1")
}
