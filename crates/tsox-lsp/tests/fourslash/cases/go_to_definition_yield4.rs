use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineGoToDefinition"]
#[test]
fn go_to_definition_yield4() {
    let content = r#"function* gen() {
    class C { [/*start*/yield 10]() {} }
}"#;
    let mut s = Session::new_for_test("goToDefinitionYield4", content);
    fourslash::unsupported("VerifyBaselineGoToDefinition"); // f.VerifyBaselineGoToDefinition(t, true, "start")
}
