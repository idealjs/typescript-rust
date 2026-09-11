use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_in_type_literal_in_type_parameter21() {
    let content = r#"class Foo<T extends ('one' | 2)[]> {}
function foo<T extends ('one' | 2)[]>() {}

type A = Foo<[/*0*/]>;
new Foo<[/*1*/]>();
foo<[/*2*/]>();
foo<[/*3*/]>;
Foo<[/*4*/]>;"#;
    let mut s = Session::new_for_test("completionListInTypeLiteralInTypeParameter21", content);
    // TODO: f.VerifyCompletions(t, "0", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, "3", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, "4", &fourslash.CompletionsExpectedList{
}
