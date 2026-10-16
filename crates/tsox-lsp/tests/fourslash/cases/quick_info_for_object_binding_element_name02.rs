use tsox_lsp::fourslash::{self, Session};


#[test]
fn quick_info_for_object_binding_element_name02() {
    let content = r#"interface I {
    property1: number;
    property2: string;
}

var foo: I;
var { property1: /**/prop1 } = foo;"#;
    let mut s = Session::new_for_test("quickInfoForObjectBindingElementName02", content);
    fourslash::verify_quick_info_at(&mut s, "", "var prop1: number", "");
}
