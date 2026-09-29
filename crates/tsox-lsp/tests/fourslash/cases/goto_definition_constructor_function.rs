use tsox_lsp::fourslash::Session;


#[test]
fn goto_definition_constructor_function() {
    let content = r#"// @allowJs: true
// @checkJs: true
// @noEmit: true
// @filename: gotoDefinitionConstructorFunction.js
function /*end*/StringStreamm() {
}
StringStreamm.prototype = {
};

function runMode () {
new [|/*start*/StringStreamm|]()
};"#;
    let _s = Session::new_for_test("gotoDefinitionConstructorFunction", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "start")
}
