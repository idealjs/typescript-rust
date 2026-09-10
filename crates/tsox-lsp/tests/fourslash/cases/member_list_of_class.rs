use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn member_list_of_class() {
    let content = r#"class C1 {
   public pubMeth() { }
   private privMeth() { }
   public pubProp = 0;
   private privProp = 0;
}
var f = new C1();
f./**/"#;
    let mut s = Session::new_for_test("memberListOfClass", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
