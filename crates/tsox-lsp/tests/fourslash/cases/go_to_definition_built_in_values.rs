use tsox_lsp::fourslash::Session;


#[test]
fn go_to_definition_built_in_values() {
    let content = r#"var u = /*undefined*/undefined;
var n = /*null*/null;
var a = function() { return /*arguments*/arguments; };
var t = /*true*/true;
var f = /*false*/false;"#;
    let _s = Session::new_for_test("goToDefinitionBuiltInValues", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, f.MarkerNames()...)
}
