use tsox_lsp::fourslash::{self, Session};


#[test]
fn find_all_refs_object_binding_element_property_name10() {
    let content = r#"interface Recursive {
    /*1*/next?: Recursive;
    value: any;
}

function f (/*2*/{ /*3*/next: { /*4*/next: x} }: Recursive) {
}"#;
    let mut s = Session::new_for_test("findAllRefsObjectBindingElementPropertyName10", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2", "3", "4")
}
