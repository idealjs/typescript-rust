use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_overriding_method9() {
    let content = r#"// @strict: false
// @Filename: a.ts
// @newline: LF
interface IFoo {
    a?: number;
    b?(x: number): void;
}
class Foo implements IFoo {
    /**/
}"#;
    let mut s = Session::new_for_test("completionsOverridingMethod9", content);
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
