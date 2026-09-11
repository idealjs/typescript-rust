use tsox_lsp::fourslash::{self, Session};


#[test]
fn go_to_type_definition_module() {
    let content = r#"// @Filename: module1.ts
module /*definition*/M {
    export var p;
}
var m: typeof M;
// @Filename: module3.ts
/*reference1*/M;
/*reference2*/m;"#;
    let mut s = Session::new_for_test("goToTypeDefinitionModule", content);
    // TODO: f.VerifyBaselineGoToTypeDefinition(t, "reference1", "reference2")
}
