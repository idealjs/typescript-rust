use tsox_lsp::fourslash::{self, Session};


#[test]
fn exhaustive_case_completions10() {
    let content = r#"
declare const u: "$1" | "2";
switch (u) {
    case/*1*/
}"#;
    // TODO: capabilities := fourslash.GetDefaultCapabilities()
    // TODO: capabilities.TextDocument.Completion.CompletionItem.SnippetSupport = new(false)
    let mut s = Session::new_with_capabilities(content, None);
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
}
