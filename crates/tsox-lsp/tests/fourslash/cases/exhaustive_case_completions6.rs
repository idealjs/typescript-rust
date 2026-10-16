use tsox_lsp::fourslash::{self, Session};


#[test]
fn exhaustive_case_completions6() {
    let content = r#"// @newline: LF
declare const p: 'A' | 'B' | 'C';

switch (p) {
    /*1*/
}"#;
    let mut s = Session::new_with_capabilities(content, None);
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
}
