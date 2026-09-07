use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.GoToEOF"]
#[test]
fn completion_list_at_eof() {
    let content = r#"var a;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("GoToEOF"); // f.GoToEOF(t)
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, nil, &fourslash.CompletionsExpectedList{
    fourslash::unsupported("InsertLine"); // f.InsertLine(t, "")
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, nil, &fourslash.CompletionsExpectedList{
    fourslash::unsupported("InsertLine"); // f.InsertLine(t, "")
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, nil, &fourslash.CompletionsExpectedList{
}
