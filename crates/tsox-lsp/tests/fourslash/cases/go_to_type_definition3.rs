use tsox_lsp::fourslash::{self, Session};


#[test]
fn go_to_type_definition3() {
    let content = r#"type /*definition*/T = string;
const x: /*reference*/T;"#;
    let mut s = Session::new_for_test("goToTypeDefinition3", content);
    // TODO: f.VerifyBaselineGoToTypeDefinition(t, "reference")
}
