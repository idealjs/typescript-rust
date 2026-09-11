use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_overriding_method3() {
    let content = r#"// @newline: LF
// @Filename: boo.d.ts
interface Ghost {
    boo(): string;
}

declare class Poltergeist implements Ghost {
    /*b*/
}"#;
    let mut s = Session::new_for_test("completionsOverridingMethod3", content);
    // TODO: f.VerifyCompletions(t, "b", &fourslash.CompletionsExpectedList{
}
