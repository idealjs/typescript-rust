use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_in_object_literal8() {
    let content = r#"declare function test<
  Variants extends Partial<Record<'hover' | 'pressed', string>>,
>(v: Variants): void

test({
  hover: "",
  /**/
});"#;
    let mut s = Session::new_for_test("completionListInObjectLiteral8", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
