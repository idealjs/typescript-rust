use tsox_lsp::fourslash::Session;


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn completion_in_function_like_body_includes_primitive_types() {
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
    let _s = Session::new_for_test("completionInFunctionLikeBody_includesPrimitiveTypes", content);
    // TODO: f.VerifyCompletions(t, []string{"1"}, &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, []string{"2", "3"}, &fourslash.CompletionsExpectedList{
}
