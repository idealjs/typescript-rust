use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.Paste"]
#[test]
fn auto_formatting_on_pasting() {
    let content = r#"namespace TestModule {
/**/
}"#;
    let mut s = Session::new_for_test("autoFormattingOnPasting", content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("Paste"); // f.Paste(t, " class TestClass{\nprivate   foo;\npublic testMethod( )\n{}\n}")
    fourslash::verify_current_file_content(&mut s, r#"namespace TestModule {
    class TestClass {
        private foo;
        public testMethod() { }
    }
}"#);
}
