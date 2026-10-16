use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_on_method_parameter_name() {
    let content = r#"class A {
    foo(nu/**/: number) {
    }
}"#;
    let mut s = Session::new_for_test("completionListOnMethodParameterName", content);
    fourslash::verify_completions_empty_at(&mut s, Some(""));
}
