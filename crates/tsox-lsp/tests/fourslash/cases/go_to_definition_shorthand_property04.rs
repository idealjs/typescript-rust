use tsox_lsp::fourslash::Session;


#[test]
fn go_to_definition_shorthand_property04() {
    let content = r#"interface Foo {
    /*2*/foo(): void
}

let x: Foo = {
    [|f/*1*/oo|]
}"#;
    let _s = Session::new_for_test("goToDefinitionShorthandProperty04", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "1")
}
