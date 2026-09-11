use tsox_lsp::fourslash::{self, Session};


#[test]
fn go_to_definition_await4() {
    let content = r#"async function outerAsyncFun() {
    let /*end*/af = async () => {
      [|/*start*/await|] Promise.resolve(0);
    }
}"#;
    let mut s = Session::new_for_test("goToDefinitionAwait4", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "start")
}
