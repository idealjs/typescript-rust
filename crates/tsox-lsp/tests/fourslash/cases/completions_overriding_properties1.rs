use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_overriding_properties1() {
    let content = r#"// @newline: LF
// @Filename: a.ts
class Base {
    protected foo: string = "bar";
}

class Sub extends Base {
    /*a*/
}"#;
    let mut s = Session::new_for_test("completionsOverridingProperties1", content);
    fourslash::go_to_marker(&mut s, "a");
    // TODO: f.VerifyCompletions(t, "a", &fourslash.CompletionsExpectedList{
}
