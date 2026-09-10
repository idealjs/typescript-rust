use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: }"]
#[test]
fn completion_property_shorthand_for_object_literal4() {
    let content = r#"// @lib: es5
const foo = 1;
const bar = 2;
const obj: any = {
  foo b/*1*/"#;
    let mut s = Session::new_for_test("completionPropertyShorthandForObjectLiteral4", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, []string{"1"}, &fourslash.CompletionsExpectedList{
    // TODO: }
}
