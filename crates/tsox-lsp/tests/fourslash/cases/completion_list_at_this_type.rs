use tsox_lsp::fourslash::{self, Session};


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
    let mut s = Session::new_for_test("completionListAtThisType", content);
    // TODO: f.VerifyCompletions(t, []string{"1", "2"}, &fourslash.CompletionsExpectedList{
}
