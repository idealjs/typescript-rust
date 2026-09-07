use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_list_of_union() {
    let content = r#"// @strictNullChecks: true
const x: { a: number, b: number } | { a: string, c: string } | { b: boolean } | number | null | undefined = { /*x*/ };
interface I { a: number; }
function f(...args: Array<I | I[]>) {}
f({ /*f*/ });"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "x", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "f", &fourslash.CompletionsExpectedList{
}
