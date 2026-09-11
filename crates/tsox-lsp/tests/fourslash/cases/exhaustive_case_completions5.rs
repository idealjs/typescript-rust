use tsox_lsp::fourslash::{self, Session};


#[test]
fn exhaustive_case_completions5() {
    let content = r#"// @newline: LF
enum P {
    " Space",
    Bar,
}

declare const p: P;

switch (p) {
    /*1*/
}"#;
    let mut s = Session::new_with_capabilities(content, None);
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
}
