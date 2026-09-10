use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_list_in_named_class_expression() {
    let content = r#"var x = class myClass {
   getClassName (){
       m/*0*/
   }
   /*1*/
}"#;
    let mut s = Session::new_for_test("completionListInNamedClassExpression", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "0", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
}
