use tsox_lsp::fourslash::Session;


#[test]
fn go_to_definition_union_type_property3() {
    let content = r#"interface Array<T> {
    /*definition*/specialPop(): T
}

var strings: string[];
var numbers: number[];

var x = (strings || numbers).[|/*usage*/specialPop|]()"#;
    let _s = Session::new_for_test("goToDefinitionUnionTypeProperty3", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "usage")
}
