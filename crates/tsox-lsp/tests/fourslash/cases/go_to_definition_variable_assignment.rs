use tsox_lsp::fourslash::{self, Session};


#[test]
fn go_to_definition_variable_assignment() {
    let content = r#"// @allowJs: true
// @checkJs: true
// @filename: foo.js
const Bar;
const Foo = /*def*/Bar = function () {}
Foo.prototype.bar = function() {}
new [|Foo/*ref*/|]();"#;
    let mut s = Session::new_for_test("goToDefinitionVariableAssignment", content);
    fourslash::go_to_file(&mut s, "foo.js");
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "ref")
}
