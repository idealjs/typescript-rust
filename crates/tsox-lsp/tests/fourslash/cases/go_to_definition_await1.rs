use tsox_lsp::fourslash::{self, Session};


#[test]
fn go_to_definition_await1() {
    let content = r#"async function /*end1*/foo() {
    [|/*start1*/await|] Promise.resolve(0);
}
function notAsync() {
    [|/*start2*/await|] Promise.resolve(0);
}"#;
    let mut s = Session::new_for_test("goToDefinitionAwait1", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "start1", "start2")
}
