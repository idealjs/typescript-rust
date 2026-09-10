use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
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
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, []string{"a", "b"}, nil)
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, []string{"c", "d"}, &fourslash.CompletionsExpectedList{
    fourslash::verify_completions_exact_at(&mut s, Some("e"), &["baseMethod"]);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "f", &fourslash.CompletionsExpectedList{
}
