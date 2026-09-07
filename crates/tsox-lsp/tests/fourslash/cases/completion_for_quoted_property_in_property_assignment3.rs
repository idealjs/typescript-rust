use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
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
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "0", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
}
