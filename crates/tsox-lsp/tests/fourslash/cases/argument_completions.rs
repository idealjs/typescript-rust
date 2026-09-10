use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn argument_completions() {
    let content = r#"
function foo(a: "a", b: "b") {}
foo("a", /*1*/);


const t3 = ['x', 'y', 'z'] as const;
const x: [string, string, string, 'a' | 'b'] = [...t3, /*2*/];
"#;
    let mut s = Session::new_for_test("argumentCompletions", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
}
