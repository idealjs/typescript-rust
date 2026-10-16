use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_class_private_fields() {
    let content = r#"class A {
    #private = 1;
}

class B extends A {
    /**/
}"#;
    let mut s = Session::new_for_test("completionListClassPrivateFields", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
