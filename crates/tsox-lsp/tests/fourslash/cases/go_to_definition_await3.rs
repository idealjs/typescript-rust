use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineGoToDefinition"]
#[test]
fn go_to_definition_await3() {
    let content = r#"class C {
    notAsync() {
      [|/*start1*/await|] Promise.resolve(0);
    }

    async /*end2*/foo() {
      [|/*start2*/await|] Promise.resolve(0);
    }
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineGoToDefinition"); // f.VerifyBaselineGoToDefinition(t, true, "start1", "start2")
}
