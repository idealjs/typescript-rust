use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_in_object_binding_pattern01() {
    let content = r#"interface I {
    property1: number;
    property2: string;
}

var foo: I;
var { /**/ } = foo;"#;
    let mut s = Session::new_for_test("completionListInObjectBindingPattern01", content);
    fourslash::verify_completions_exact_at(&mut s, Some(""), &["property1", "property2"]);
}
