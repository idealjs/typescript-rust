use tsox_lsp::fourslash::Session;


#[test]
fn go_to_definition_constructor_of_class_when_class_is_preceded_by_namespace01() {
    let content = r#"namespace Foo {
    export var x;
}

class Foo {
    /*definition*/constructor() {
    }
}

var x = new [|/*usage*/Foo|]();"#;
    let _s = Session::new_for_test("goToDefinitionConstructorOfClassWhenClassIsPrecededByNamespace01", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "usage")
}
