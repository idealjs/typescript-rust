use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
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
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", nil)
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
}
