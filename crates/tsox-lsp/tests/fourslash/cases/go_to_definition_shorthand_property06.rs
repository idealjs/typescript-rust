use tsox_lsp::fourslash::Session;


#[test]
fn go_to_definition_shorthand_property06() {
    let content = r#"interface Foo {
    /*2*/foo(): void
}
const foo = 1;
let x: Foo = {
    [|f/*1*/oo|]()
}"#;
    let _s = Session::new_for_test("goToDefinitionShorthandProperty06", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "1")
}
