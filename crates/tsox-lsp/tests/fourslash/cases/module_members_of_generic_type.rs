use tsox_lsp::fourslash::{self, Session};


#[test]
fn module_members_of_generic_type() {
    let content = r#"namespace M {
    export var x = <T>(x: T) => x;
}
var r = M./**/;"#;
    let mut s = Session::new_for_test("moduleMembersOfGenericType", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
