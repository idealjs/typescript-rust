use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineGoToDefinition"]
#[test]
fn go_to_definition_undefined_symbols() {
    let content = r#"some/*undefinedValue*/Variable;
var a: some/*undefinedType*/Type;
var x = {}; x.some/*undefinedProperty*/Property;
var a: any; a.some/*unkownProperty*/Property;"#;
    let mut s = Session::new_for_test("goToDefinitionUndefinedSymbols", content);
    fourslash::unsupported("VerifyBaselineGoToDefinition"); // f.VerifyBaselineGoToDefinition(t, true, f.MarkerNames()...)
}
