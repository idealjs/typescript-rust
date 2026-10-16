use tsox_lsp::fourslash::{self, Session};


#[test]
fn deduplicate_duplicate_merged_bind_check_errors() {
    let content = r#"class X {
  foo() {
      return 1;
  }
  get foo() {
      return 1;
  }
}"#;
    let mut s = Session::new_for_test("deduplicateDuplicateMergedBindCheckErrors", content);
    fourslash::verify_number_of_errors_in_current_file(&mut s, 2);
}
