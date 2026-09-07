use tsox_lsp::fourslash::{self, Session};

#[test]
fn quick_info_for_object_binding_element_name01() {
    let content = r#"interface I {
    property1: number;
    property2: string;
}

var foo: I;
var { /**/property1 } = foo;"#;
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::verify_quick_info_at(&mut s, "", "var property1: number", "");
}
