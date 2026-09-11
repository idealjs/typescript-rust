use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_generator_functions() {
    let content = r#"function /*a*/ ;
function* /*b*/ ;
interface I {
    abstract baseMethod(): Iterable<number>;
}
class C implements I {
    */*c*/ ;
    public */*d*/
}
const o: I = {
    */*e*/
};
1 * /*f*/"#;
    let mut s = Session::new_for_test("completionsGeneratorFunctions", content);
    // TODO: f.VerifyCompletions(t, []string{"a", "b"}, nil)
    // TODO: f.VerifyCompletions(t, []string{"c", "d"}, &fourslash.CompletionsExpectedList{
    fourslash::verify_completions_exact_at(&mut s, Some("e"), &["baseMethod"]);
    fourslash::go_to_marker(&mut s, "f");
    // TODO: f.VerifyCompletions(t, "f", &fourslash.CompletionsExpectedList{
}
