use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completions_for_latter_type_parameters_in_constraints1() {
    let content = r#"// https://github.com/microsoft/TypeScript/issues/56474
function test<First extends S/*1*/, Second>(a: First, b: Second) {}
type A1<K extends /*2*/, L> = K"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, []string{"1"}, &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, []string{"2"}, &fourslash.CompletionsExpectedList{
}
