use tsox_lsp::fourslash::{self, Session};


#[test]
fn exhaustive_case_completions11() {
    let content = r#"
declare const u: "$1" | "2";
switch (u) {
    case/*1*/
}"#;
    let mut s = Session::new_with_capabilities(content, None);
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
}
