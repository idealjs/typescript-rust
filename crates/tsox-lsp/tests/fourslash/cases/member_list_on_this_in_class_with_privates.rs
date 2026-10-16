use tsox_lsp::fourslash::{self, Session};


#[test]
fn member_list_on_this_in_class_with_privates() {
    let content = r#"class C1 {
   public pubMeth() {this./**/} // test on 'this.'
   private privMeth() {}
   public pubProp = 0;
   private privProp = 0;
}"#;
    let mut s = Session::new_for_test("memberListOnThisInClassWithPrivates", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
