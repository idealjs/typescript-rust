use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_for_quoted_property_in_property_assignment2() {
    let content = r#"export interface Config {
   files: ConfigFiles
}
export interface ConfigFiles {
  jspm: string;
  'jspm:browser': string;
}
let config: Config;
config = {
   files: {
       /*0*/: '',
       '[|/*1*/|]': ''
   }
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "0", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
}
