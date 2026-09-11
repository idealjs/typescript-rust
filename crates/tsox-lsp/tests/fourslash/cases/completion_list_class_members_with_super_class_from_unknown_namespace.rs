use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_class_members_with_super_class_from_unknown_namespace() {
    let content = r#"class Child extends Namespace.Parent {
    /**/
}"#;
    let mut s = Session::new_for_test("completionListClassMembersWithSuperClassFromUnknownNamespace", content);
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
