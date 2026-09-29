use tsox_lsp::fourslash::{self, Session};


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
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
