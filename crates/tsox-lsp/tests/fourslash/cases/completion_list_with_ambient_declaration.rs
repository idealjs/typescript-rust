use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_with_ambient_declaration() {
    let content = r#"declare module "http" {
   var x;
   /*1*/
}
declare module 'https' {
}
/*2*/"#;
    let mut s = Session::new_for_test("completionListWithAmbientDeclaration", content);
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
}
