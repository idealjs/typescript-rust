use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_property_shorthand_for_object_literal2() {
    let content = r#"// @lib: es5
const foo = 1;
const bar = 2;
const obj1 = {
  foo b/*1*/
};
const obj2: any = {
  foo b/*2*/
};"#;
    let mut s = Session::new_for_test("completionPropertyShorthandForObjectLiteral2", content);
    // TODO: f.VerifyCompletions(t, []string{"1"}, &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, []string{"2"}, &fourslash.CompletionsExpectedList{
}
