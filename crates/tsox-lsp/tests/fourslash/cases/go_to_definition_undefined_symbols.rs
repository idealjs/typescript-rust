use tsox_lsp::fourslash::Session;


#[test]
fn go_to_definition_undefined_symbols() {
    let content = r#"some/*undefinedValue*/Variable;
var a: some/*undefinedType*/Type;
var x = {}; x.some/*undefinedProperty*/Property;
var a: any; a.some/*unkownProperty*/Property;"#;
    let _s = Session::new_for_test("goToDefinitionUndefinedSymbols", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, f.MarkerNames()...)
}
