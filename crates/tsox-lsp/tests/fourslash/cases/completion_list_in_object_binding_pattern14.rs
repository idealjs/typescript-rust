use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_in_object_binding_pattern14() {
    let content = r#"const { b/**/ } = new class {
    private ab;
    protected bc;
}"#;
    let mut s = Session::new_for_test("completionListInObjectBindingPattern14", content);
    fourslash::verify_completions_empty_at(&mut s, Some(""));
}
