use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_list_object_members() {
    let content = r#" var object: {
     (bar: any): any;
     new (bar: any): any;
     [bar: any]: any;
     bar: any;
     foo(bar: any): any;
 };
object./**/"#;
    let mut s = Session::new_for_test("completionListObjectMembers", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
