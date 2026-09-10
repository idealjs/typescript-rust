use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_list_in_type_literal_in_type_parameter17() {
    let content = r#"class Foo<T extends { x: 'one' | 2 }> {}
function foo<T extends { x: 'one' | 2 }>() {}

type A = Foo<{ x: /*0*/ }>;
new Foo<{ x: /*1*/ }>();
foo<{ x: /*2*/ }>();
foo<{ x: /*3*/ }>;
Foo<{ x: /*4*/ }>;"#;
    let mut s = Session::new_for_test("completionListInTypeLiteralInTypeParameter17", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "0", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "3", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "4", &fourslash.CompletionsExpectedList{
}
