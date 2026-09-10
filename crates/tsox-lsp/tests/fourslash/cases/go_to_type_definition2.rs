use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineGoToTypeDefinition"]
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
    let mut s = Session::new_for_test("goToTypeDefinition2", content);
    fourslash::unsupported("VerifyBaselineGoToTypeDefinition"); // f.VerifyBaselineGoToTypeDefinition(t, "reference")
}
