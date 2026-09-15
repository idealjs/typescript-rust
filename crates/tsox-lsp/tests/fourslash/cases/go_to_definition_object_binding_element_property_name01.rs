use tsox_lsp::fourslash::Session;


#[test]
fn go_to_definition_object_binding_element_property_name01() {
    let content = r#"interface I {
    /*def*/property1: number;
    property2: string;
}

var foo: I;
var { [|/*use*/property1|]: prop1 } = foo;"#;
    let _s = Session::new_for_test("goToDefinitionObjectBindingElementPropertyName01", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "use")
}
