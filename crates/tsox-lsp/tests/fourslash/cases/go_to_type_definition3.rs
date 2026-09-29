use tsox_lsp::fourslash::Session;


#[test]
fn go_to_type_definition3() {
    let content = r#"type /*definition*/T = string;
const x: /*reference*/T;"#;
    let _s = Session::new_for_test("goToTypeDefinition3", content);
    // TODO: f.VerifyBaselineGoToTypeDefinition(t, "reference")
}
