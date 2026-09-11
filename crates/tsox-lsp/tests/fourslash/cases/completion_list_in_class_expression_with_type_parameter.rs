use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_in_class_expression_with_type_parameter() {
    let content = r#"var x = class myClass <TypeParam> {
   getClassName (){
       /*0*/
       var tmp: /*0Type*/;
   }
   prop: Ty/*1*/
}"#;
    let mut s = Session::new_for_test("completionListInClassExpressionWithTypeParameter", content);
    // TODO: f.VerifyCompletions(t, "0", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, []string{"0Type", "1"}, &fourslash.CompletionsExpectedList{
}
