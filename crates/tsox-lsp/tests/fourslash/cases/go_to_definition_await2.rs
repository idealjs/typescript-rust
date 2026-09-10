use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineGoToDefinition"]
#[test]
fn go_to_definition_await2() {
    let content = r#"[|/*start*/await|] Promise.resolve(0);"#;
    let mut s = Session::new_for_test("goToDefinitionAwait2", content);
    fourslash::unsupported("VerifyBaselineGoToDefinition"); // f.VerifyBaselineGoToDefinition(t, true, "start")
}
