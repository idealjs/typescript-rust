use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_overriding_method17() {
    let content = r#"// @Filename: a.ts
// @newline: LF
interface Interface {
    method(): void;
}

export class Class implements Interface {
    property = "yadda";

    /**/
}"#;
    let mut s = Session::new_for_test("completionsOverridingMethod17", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
