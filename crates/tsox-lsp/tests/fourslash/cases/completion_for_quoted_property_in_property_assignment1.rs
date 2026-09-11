use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_for_quoted_property_in_property_assignment1() {
    let content = r#"export interface Configfiles {
  jspm: string;
  'jspm:browser': string;
}
let files: Configfiles;
files = {
   /*0*/: '',
   '[|/*1*/|]': ''
}"#;
    let mut s = Session::new_for_test("completionForQuotedPropertyInPropertyAssignment1", content);
    fourslash::go_to_marker(&mut s, "0");
    // TODO: f.VerifyCompletions(t, "0", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
}
