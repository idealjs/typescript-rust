use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyQuickInfoExists"]
#[test]
fn symbol_name_at_unparseable_function_overload() {
    let content = r#"class TestClass {
    public function foo(x: string): void;
    public function foo(): void;
    foo(x: any): void {
        this.bar(/**/x); // should not error
    }
}
"#;
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("VerifyQuickInfoExists"); // f.VerifyQuickInfoExists(t)
}
