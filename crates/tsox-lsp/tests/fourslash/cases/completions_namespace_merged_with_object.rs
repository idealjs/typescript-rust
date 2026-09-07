use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completions_namespace_merged_with_object() {
    let content = r#"namespace N {
    export type T = number;
}
const N = { m() {} };
let x: N./*type*/;
N./*value*/;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "type", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "value", &fourslash.CompletionsExpectedList{
}
