use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_class_member_after_js_doc_with_invalid_js_doc_tag_in_the_comment1() {
    let content = r#"export class NeedsPrefix {
  private _prefixes: {
    add: Record<string, any>;
    browsers: {selected: string[]};
  };

  constructor(browsers: string[]) {
  }

  /** Checks whether an @-rule needs to be prefixed. */
  /**/atRule(identifier: string): boolean {
    return true;
  }
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
