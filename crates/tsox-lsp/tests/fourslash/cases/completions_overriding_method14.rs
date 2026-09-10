use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completions_overriding_method14() {
    let content = r#"// @Filename: a.ts
// @strictNullChecks: true
// @newline: LF
interface IFoo {
    foo?(arg: string): number;
}
class Foo implements IFoo {
    /**/
}"#;
    let mut s = Session::new_for_test("completionsOverridingMethod14", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
