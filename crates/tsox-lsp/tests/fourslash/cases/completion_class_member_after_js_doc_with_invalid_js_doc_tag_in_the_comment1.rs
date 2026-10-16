use tsox_lsp::fourslash::{self, Session};


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
    let mut s = Session::new_for_test("completionClassMemberAfterJSDocWithInvalidJSDocTagInTheComment1", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
