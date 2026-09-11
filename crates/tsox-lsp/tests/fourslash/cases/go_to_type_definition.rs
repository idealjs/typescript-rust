use tsox_lsp::fourslash::{self, Session};


#[test]
fn go_to_type_definition() {
    let content = r#"// @Filename: goToTypeDefinition_Definition.ts
class /*definition*/C {
    p;
}
var c: C;
// @Filename: goToTypeDefinition_Consumption.ts
/*reference*/c = undefined;"#;
    let mut s = Session::new_for_test("goToTypeDefinition", content);
    // TODO: f.VerifyBaselineGoToTypeDefinition(t, "reference")
}
