use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_ecma_private_member_trigger_character() {
    let content = r#"// @target: esnext
class K {
  #value: number;

  foo() {
     this.#/**/
  }
}"#;
    let mut s = Session::new_for_test("completionsECMAPrivateMemberTriggerCharacter", content);
    fourslash::verify_completions_exact_at(&mut s, Some(""), &["#value", "foo"]);
    fourslash::verify_completions_exact_at(&mut s, Some(""), &["#value", "foo"]);
}
