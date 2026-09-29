use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_object_literal_method6() {
    let content = r#"// @Filename: a.ts
type T = {
    foo: () => Promise<void>;
}
const foo: T = {
    async f/**/
}"#;
    let mut s = Session::new_for_test("completionsObjectLiteralMethod6", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
