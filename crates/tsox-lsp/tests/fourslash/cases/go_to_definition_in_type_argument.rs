use tsox_lsp::fourslash::Session;


#[test]
fn go_to_definition_in_type_argument() {
    let content = r#"class /*fooDefinition*/Foo<T> { }

class /*barDefinition*/Bar { }

var x = new Fo/*fooReference*/o<Ba/*barReference*/r>();"#;
    let _s = Session::new_for_test("goToDefinitionInTypeArgument", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, false, "barReference", "fooReference")
}
