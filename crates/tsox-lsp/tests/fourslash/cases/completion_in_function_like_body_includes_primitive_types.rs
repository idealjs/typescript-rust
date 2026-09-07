use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
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
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, []string{"1"}, &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, []string{"2", "3"}, &fourslash.CompletionsExpectedList{
}
