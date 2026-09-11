use tsox_lsp::fourslash::{self, Session};


#[test]
fn member_list_error_recovery() {
    let content = r#"class Foo { static fun() { }; }

Foo./**/;
/*1*/var bar;"#;
    let mut s = Session::new_for_test("memberListErrorRecovery", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
