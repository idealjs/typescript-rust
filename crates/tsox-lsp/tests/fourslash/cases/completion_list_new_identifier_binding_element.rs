use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_new_identifier_binding_element() {
    let content = r#"var { x:html/*1*/"#;
    let mut s = Session::new_for_test("completionListNewIdentifierBindingElement", content);
    fourslash::verify_completions_empty_at(&mut s, Some("1"));
    // TODO: }
}
