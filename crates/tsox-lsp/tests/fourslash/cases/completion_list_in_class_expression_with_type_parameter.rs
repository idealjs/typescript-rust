use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_list_in_class_expression_with_type_parameter() {
    let content = r#"var x = class myClass <TypeParam> {
   getClassName (){
       /*0*/
       var tmp: /*0Type*/;
   }
   prop: Ty/*1*/
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "0", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, []string{"0Type", "1"}, &fourslash.CompletionsExpectedList{
}
