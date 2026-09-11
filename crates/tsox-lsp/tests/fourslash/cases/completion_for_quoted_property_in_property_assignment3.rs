use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_for_quoted_property_in_property_assignment3() {
    let content = r#" let configFiles1: {
     jspm: string;
     'jspm:browser': string;
 } = {
         /*0*/: "",
 }
 let configFiles2: {
     jspm: string;
     'jspm:browser': string;
 } = {
        jspm: "",
        '[|/*1*/|]': ""
 }"#;
    let mut s = Session::new_for_test("completionForQuotedPropertyInPropertyAssignment3", content);
    // TODO: f.VerifyCompletions(t, "0", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
}
