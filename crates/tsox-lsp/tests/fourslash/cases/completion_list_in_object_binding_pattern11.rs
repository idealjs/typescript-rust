use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_in_object_binding_pattern11() {
    let content = r#"interface I {
    property1: number;
    property2: string;
}

var { property1: prop1, /**/ }: I;"#;
    let mut s = Session::new_for_test("completionListInObjectBindingPattern11", content);
    fourslash::verify_completions_exact_at(&mut s, Some(""), &["property2"]);
}
