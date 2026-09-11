use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_in_function_like_body_includes_primitive_types() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"class Foo<T> { }
class Bar { }
function includesTypes() {
    new Foo</*1*/
}
function excludesTypes1() {
    new Bar</*2*/
}
function excludesTypes2() {
    1</*3*/
}"#;
    let mut s = Session::new_for_test("completionInFunctionLikeBody_includesPrimitiveTypes", content);
    // TODO: f.VerifyCompletions(t, []string{"1"}, &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, []string{"2", "3"}, &fourslash.CompletionsExpectedList{
}
