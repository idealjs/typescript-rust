use tsox_lsp::fourslash::{self, Session};


#[test]
fn go_to_definition_variable_assignment2() {
    let content = r#"// @filename: foo.ts
const Bar;
const Foo = /*def*/Bar = function () {}
Foo.prototype.bar = function() {}
new [|Foo/*ref*/|]();"#;
    let mut s = Session::new_for_test("goToDefinitionVariableAssignment2", content);
    fourslash::go_to_file(&mut s, "foo.ts");
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "ref")
}
