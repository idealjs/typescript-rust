use tsox_lsp::fourslash::Session;


#[test]
fn completion_property_shorthand_for_object_literal4() {
    let content = r#"// @lib: es5
const foo = 1;
const bar = 2;
const obj: any = {
  foo b/*1*/"#;
    let _s = Session::new_for_test("completionPropertyShorthandForObjectLiteral4", content);
    // TODO: f.VerifyCompletions(t, []string{"1"}, &fourslash.CompletionsExpectedList{
    // TODO: }
}
