use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_index_signature_constraint1() {
    let content = r#"// @strict: true

repro #9900

interface Test {
  a?: number;
  b?: string;
}

interface TestIndex {
  [key: string]: Test;
}

declare function testFunc<T extends TestIndex>(t: T): void;

testFunc({
  test: {
    /**/
  },
});"#;
    let mut s = Session::new_for_test("completionsIndexSignatureConstraint1", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
