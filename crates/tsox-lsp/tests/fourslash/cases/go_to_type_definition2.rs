use tsox_lsp::fourslash::Session;


#[test]
fn go_to_type_definition2() {
    let content = r#"// @Filename: goToTypeDefinition2_Definition.ts
interface /*definition*/I1 {
    p;
}
type propertyType = I1;
interface I2 {
    property: propertyType;
}
// @Filename: goToTypeDefinition2_Consumption.ts
var i2: I2;
i2.prop/*reference*/erty;"#;
    let _s = Session::new_for_test("goToTypeDefinition2", content);
    // TODO: f.VerifyBaselineGoToTypeDefinition(t, "reference")
}
