use tsox_lsp::fourslash::Session;


#[test]
fn go_to_definition_built_in_types() {
    let content = r#"var n: /*number*/number;
var s: /*string*/string;
var b: /*boolean*/boolean;
var v: /*void*/void;"#;
    let _s = Session::new_for_test("goToDefinitionBuiltInTypes", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, f.MarkerNames()...)
}
