use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn import_meta_completion_details() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @filename: index.mts
// @module: Node16
// @strict: true
let x = import.meta/**/;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyNoErrors"); // f.VerifyNoErrors(t)
}
