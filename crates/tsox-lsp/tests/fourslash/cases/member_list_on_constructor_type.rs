use tsox_lsp::fourslash::{self, Session};


#[test]
fn member_list_on_constructor_type() {
    let content = r#"// @lib: es5
var f: new () => void;
f./*1*/"#;
    let mut s = Session::new_for_test("memberListOnConstructorType", content);
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
}
