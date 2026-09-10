use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_list_class_private_fields() {
    let content = r#"class A {
    #private = 1;
}

class B extends A {
    /**/
}"#;
    let mut s = Session::new_for_test("completionListClassPrivateFields", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
