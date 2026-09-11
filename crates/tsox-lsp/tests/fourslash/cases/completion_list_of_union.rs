use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_of_union() {
    let content = r#"// @strictNullChecks: true
const x: { a: number, b: number } | { a: string, c: string } | { b: boolean } | number | null | undefined = { /*x*/ };
interface I { a: number; }
function f(...args: Array<I | I[]>) {}
f({ /*f*/ });"#;
    let mut s = Session::new_for_test("completionListOfUnion", content);
    // TODO: f.VerifyCompletions(t, "x", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, "f", &fourslash.CompletionsExpectedList{
}
