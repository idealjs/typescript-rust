use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineGoToDefinition"]
#[test]
fn go_to_definition_return5() {
    let content = r#"function foo() {
    class Foo {
        static { [|/*start*/return|]; }
    }
}"#;
    let mut s = Session::new_for_test("goToDefinitionReturn5", content);
    fourslash::unsupported("VerifyBaselineGoToDefinition"); // f.VerifyBaselineGoToDefinition(t, true, "start")
}
