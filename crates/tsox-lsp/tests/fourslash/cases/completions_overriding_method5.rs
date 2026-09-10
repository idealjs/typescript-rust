use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completions_overriding_method5() {
    let content = r#"// @newline: LF
// @Filename: a.ts
abstract class Ab {
    abstract met(n: string): void;
    met2(n: number): void {
        return;
    }
}

abstract class Abc extends Ab {
    /*a*/
    abstract /*b*/
    abstract [|m/*c*/|]
}"#;
    let mut s = Session::new_for_test("completionsOverridingMethod5", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "a", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "b", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "c", &fourslash.CompletionsExpectedList{
}
