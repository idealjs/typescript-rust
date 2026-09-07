use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineGoToDefinition"]
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
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineGoToDefinition"); // f.VerifyBaselineGoToDefinition(t, true, "start")
}
