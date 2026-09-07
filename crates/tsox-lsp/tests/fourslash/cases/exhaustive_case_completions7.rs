use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn exhaustive_case_completions7() {
    let content = r#"// @newline: LF
export function foo(position: -1 | 0 | 1) {
    switch (position) {
        /**/
    }
}"#;
    let mut s = Session::new_with_capabilities(content, None);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
