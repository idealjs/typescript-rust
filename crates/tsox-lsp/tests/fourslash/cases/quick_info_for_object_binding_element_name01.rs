use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyQuickInfoAt"]
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
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "", "var property1: number", "")
}
