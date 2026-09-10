use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineGoToDefinition"]
#[test]
fn go_to_definition_yield2() {
    let content = r#"function* outerGen() {
    function* /*end*/gen() {
        [|/*start*/yield|] 0;
    }
    return gen
}"#;
    let mut s = Session::new_for_test("goToDefinitionYield2", content);
    fourslash::unsupported("VerifyBaselineGoToDefinition"); // f.VerifyBaselineGoToDefinition(t, true, "start")
}
