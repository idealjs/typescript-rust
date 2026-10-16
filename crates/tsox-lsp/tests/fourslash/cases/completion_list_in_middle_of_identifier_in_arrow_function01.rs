use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_in_middle_of_identifier_in_arrow_function01() {
    let content = r#"xyz => x/*1*/y"#;
    let mut s = Session::new_for_test("completionListInMiddleOfIdentifierInArrowFunction01", content);
    fourslash::verify_completions_include_exclude_at(&mut s, Some("1"), &["xyz"], &[]);
}
