use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn import_meta_completion_details() {
    let content = r#"// @filename: index.mts
// @module: Node16
// @strict: true
let x = import.meta/**/;"#;
    let mut s = Session::new_for_test("importMetaCompletionDetails", content);
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
    fourslash::verify_no_errors(&mut s, );
}
