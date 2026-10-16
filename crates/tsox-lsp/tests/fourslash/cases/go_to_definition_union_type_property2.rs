use tsox_lsp::fourslash::Session;


#[test]
fn go_to_definition_union_type_property2() {
    let content = r#"interface HasAOrB {
    /*propertyDefinition1*/a: string;
    b: string;
}

interface One {
    common: { /*propertyDefinition2*/a : number; };
}

interface Two {
    common: HasAOrB;
}

var x : One | Two;

x.common.[|/*propertyReference*/a|];"#;
    let _s = Session::new_for_test("goToDefinitionUnionTypeProperty2", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "propertyReference")
}
