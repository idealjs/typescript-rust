use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completions_on_import_identifier_with_from_on_next_line() {
    let content = r#"import something/*1*/
from"#;
    let mut s = Session::new_for_test("completionsOnImportIdentifierWithFromOnNextLine", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
}

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completions_on_import_type_with_from_on_next_line() {
    let content = r#"import type/*1*/
from"#;
    let mut s = Session::new_for_test("completionsOnImportTypeWithFromOnNextLine", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
}
