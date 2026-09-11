use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_at_beginning_of_identifier_in_arrow_function01() {
    let content = r#"xyz => /*1*/x"#;
    let mut s = Session::new_for_test("completionListAtBeginningOfIdentifierInArrowFunction01", content);
    fourslash::verify_completions_include_exclude_at(&mut s, Some("1"), &["xyz"], &[]);
}
