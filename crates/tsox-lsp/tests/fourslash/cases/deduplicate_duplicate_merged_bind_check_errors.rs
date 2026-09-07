use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyNumberOfErrorsInCurrentFile"]
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
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyNumberOfErrorsInCurrentFile"); // f.VerifyNumberOfErrorsInCurrentFile(t, 2)
}
