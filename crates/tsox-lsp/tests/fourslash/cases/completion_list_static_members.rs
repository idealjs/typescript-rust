use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_static_members() {
    let content = r#"// @lib: es5
class Foo {
    static a() {}
    static b() {}
}
Foo./**/"#;
    let mut s = Session::new_for_test("completionListStaticMembers", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
