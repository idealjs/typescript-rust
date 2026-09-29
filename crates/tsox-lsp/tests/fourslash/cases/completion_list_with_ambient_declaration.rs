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
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "2");
    // TODO: f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
}
