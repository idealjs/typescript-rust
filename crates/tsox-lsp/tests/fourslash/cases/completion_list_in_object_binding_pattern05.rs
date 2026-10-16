use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_in_object_binding_pattern05() {
    let content = r#"interface I {
    property1: number;
    property2: string;
}

var foo: I;
var { property1/**/ } = foo;"#;
    let mut s = Session::new_for_test("completionListInObjectBindingPattern05", content);
    fourslash::verify_completions_exact_at(&mut s, Some(""), &["property1", "property2"]);
}
