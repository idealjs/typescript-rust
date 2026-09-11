use tsox_lsp::fourslash::{self, Session};


#[test]
fn exhaustive_case_completions8() {
    let content = r#"// @newline: LF
export function foo(position: -1n | 0n) {
    switch (position) {
        /**/
    }
}"#;
    let mut s = Session::new_with_capabilities(content, None);
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
