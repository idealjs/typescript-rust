use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_in_named_class_expression() {
    let content = r#"var x = class myClass {
   getClassName (){
       m/*0*/
   }
   /*1*/
}"#;
    let mut s = Session::new_for_test("completionListInNamedClassExpression", content);
    fourslash::go_to_marker(&mut s, "0");
    // TODO: f.VerifyCompletions(t, "0", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
}
