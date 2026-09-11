use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_class_members() {
    let content = r#"// @lib: es5
class Class {
    private privateInstanceMethod() { }
    public publicInstanceMethod() { }

    private privateProperty = 1;
    public publicProperty = 1;

    private static privateStaticProperty = 1;
    public static publicStaticProperty = 1;

    private static privateStaticMethod() { }
    public static publicStaticMethod() {
        Class./*staticsInsideClassScope*/publicStaticMethod();
        var c = new Class();
        c./*instanceMembersInsideClassScope*/privateProperty;
    }
}

Class./*staticsOutsideClassScope*/publicStaticMethod();
var c = new Class();
c./*instanceMembersOutsideClassScope*/privateProperty;"#;
    let mut s = Session::new_for_test("completionListClassMembers", content);
    // TODO: f.VerifyCompletions(t, "staticsInsideClassScope", &fourslash.CompletionsExpectedList{
    fourslash::verify_completions_unsorted_at(&mut s, Some("instanceMembersInsideClassScope"), &["privateInstanceMethod", "publicInstanceMethod", "privateProperty", "publicProperty"]);
    // TODO: f.VerifyCompletions(t, "staticsOutsideClassScope", &fourslash.CompletionsExpectedList{
    fourslash::verify_completions_exact_at(&mut s, Some("instanceMembersOutsideClassScope"), &["publicInstanceMethod", "publicProperty"]);
}
