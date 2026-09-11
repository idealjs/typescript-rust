use tsox_lsp::fourslash::{self, Session};


#[test]
fn member_list_in_with_block() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"class c {
    static x: number;
    public foo() {
        with ({}) {
            function f() { }
            var d = this./*1*/foo;
            /*2*/
        }
    }
}"#;
    let mut s = Session::new_for_test("memberListInWithBlock", content);
    fourslash::verify_completions_empty_at(&mut s, Some("1"));
    // TODO: f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
}
