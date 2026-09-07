use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn member_list_on_this_in_class_with_privates() {
    let content = r#"class C1 {
   public pubMeth() {this./**/} // test on 'this.'
   private privMeth() {}
   public pubProp = 0;
   private privProp = 0;
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
