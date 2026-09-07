use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_list_at_this_type() {
    let content = r#"// @stableTypeOrdering: true
class Test {
    foo() {}

    bar() {
        this.baz(this, "/*1*/");

        const t = new Test()
        this.baz(t, "/*2*/");
    }

    baz<T>(a: T, k: keyof T) {}
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, []string{"1", "2"}, &fourslash.CompletionsExpectedList{
}
