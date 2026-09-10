use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineGoToDefinition"]
#[test]
fn go_to_definition_variable_assignment1() {
    let content = r#"// @allowJs: true
// @checkJs: true
// @filename: foo.js
const Foo = module./*def*/exports = function () {}
Foo.prototype.bar = function() {}
new [|Foo/*ref*/|]();"#;
    let mut s = Session::new_for_test("goToDefinitionVariableAssignment1", content);
    fourslash::go_to_file(&mut s, "foo.js");
    fourslash::unsupported("VerifyBaselineGoToDefinition"); // f.VerifyBaselineGoToDefinition(t, true, "ref")
}
