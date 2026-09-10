use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_list_super_members() {
    let content = r#"class Base {
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
class Class extends Base {
    private test() {
        super./**/
    }
}"#;
    let mut s = Session::new_for_test("completionListSuperMembers", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
