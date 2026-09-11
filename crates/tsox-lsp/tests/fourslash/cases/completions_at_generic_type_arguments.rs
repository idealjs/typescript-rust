use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_at_generic_type_arguments() {
    let content = r#"// @lib: es5
class Foo<T1, T2> {}
const foo = new Foo</*1*/, /*2*/,

function foo<T1, T2>() {}
const f = foo</*3*/, /*4*/,"#;
    let mut s = Session::new_for_test("completionsAtGenericTypeArguments", content);
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "2");
    // TODO: f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "3");
    // TODO: f.VerifyCompletions(t, "3", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "4");
    // TODO: f.VerifyCompletions(t, "4", &fourslash.CompletionsExpectedList{
}
