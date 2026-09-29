use tsox_lsp::fourslash::Session;


#[test]
fn go_to_definition_shorthand_object_literal_with_interface() {
    let content = r#"interface Something {
    [|foo|]: string;
}

function makeSomething([|foo|]: string): Something {
    return { [|f/*1*/oo|] };
}"#;
    let _s = Session::new_for_test("goToDefinitionShorthandObjectLiteralWithInterface", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "1")
}
