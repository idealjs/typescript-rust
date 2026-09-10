use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn exhaustive_case_completions11() {
    let content = r#"
declare const u: "$1" | "2";
switch (u) {
    case/*1*/
}"#;
    let mut s = Session::new_with_capabilities(content, None);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
}
