use tsox_lsp::fourslash::{self, Session};


#[test]
fn member_list_on_constructor_type() {
    let content = r#"// @lib: es5
var f: new () => void;
f./*1*/"#;
    let mut s = Session::new_for_test("memberListOnConstructorType", content);
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
}
