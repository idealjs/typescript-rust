use tsox_lsp::fourslash::{self, Session};


#[test]
fn go_to_definition_return5() {
    let content = r#"function foo() {
    class Foo {
        static { [|/*start*/return|]; }
    }
}"#;
    let mut s = Session::new_for_test("goToDefinitionReturn5", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "start")
}
