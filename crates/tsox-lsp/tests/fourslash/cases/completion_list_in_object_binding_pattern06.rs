use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_in_object_binding_pattern06() {
    let content = r#"interface I {
    property1: number;
    property2: string;
}

var foo: I;
var { property1, property2, /**/ } = foo;"#;
    let mut s = Session::new_for_test("completionListInObjectBindingPattern06", content);
    fourslash::verify_completions_empty_at(&mut s, Some(""));
}
