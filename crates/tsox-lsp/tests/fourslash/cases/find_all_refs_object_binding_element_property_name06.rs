use tsox_lsp::fourslash::Session;


#[test]
fn find_all_refs_object_binding_element_property_name06() {
    let content = r#"interface I {
    /*0*/property1: number;
    property2: string;
}

var elems: I[];
for (let { /*1*/property1: p } of elems) {
}
for (let { /*2*/property1 } of elems) {
}
for (var { /*3*/property1: p1 } of elems) {
}
var p2;
for ({ /*4*/property1 : p2 } of elems) {
}"#;
    let _s = Session::new_for_test("findAllRefsObjectBindingElementPropertyName06", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "0", "1", "3", "4", "2")
}
