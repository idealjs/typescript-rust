use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_list_in_named_class_expression_with_shadowing() {
    let content = r#"class myClass { /*0*/ }
/*1*/
var x = class myClass {
   getClassName (){
       m/*2*/
   }
   /*3*/
}
var y = class {
   getSomeName() {
       /*4*/
   }
   /*5*/
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "0", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, []string{"1", "4"}, &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, []string{"3", "5"}, &fourslash.CompletionsExpectedList{
}
