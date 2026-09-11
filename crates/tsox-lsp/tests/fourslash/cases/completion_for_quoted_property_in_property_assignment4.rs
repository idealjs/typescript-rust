use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_for_quoted_property_in_property_assignment4() {
    let content = r#"export interface ConfigFiles {
  jspm: string;
  'jspm:browser': string;
}
function foo(c: ConfigFiles) {}
foo({
    j/*0*/: "",
    "[|/*1*/|]": "",
})"#;
    let mut s = Session::new_for_test("completionForQuotedPropertyInPropertyAssignment4", content);
    // TODO: f.VerifyCompletions(t, "0", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
}
