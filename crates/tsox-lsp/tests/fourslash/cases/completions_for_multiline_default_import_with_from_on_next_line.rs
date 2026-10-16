use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_on_import_identifier_with_from_on_next_line() {
    let content = r#"import something/*1*/
from"#;
    let mut s = Session::new_for_test("completionsOnImportIdentifierWithFromOnNextLine", content);
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
}

#[test]
fn completions_on_import_type_with_from_on_next_line() {
    let content = r#"import type/*1*/
from"#;
    let mut s = Session::new_for_test("completionsOnImportTypeWithFromOnNextLine", content);
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
}
