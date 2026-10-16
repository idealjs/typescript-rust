use tsox_lsp::fourslash::Session;


#[test]
fn references_to_string_literal_value() {
    let content = r#"// @lib: es5
const s: string = "some /*1*/ string";"#;
    let _s = Session::new_for_test("referencesToStringLiteralValue", content);
    // TODO: f.MarkTestAsStradaServer()
    // TODO: f.VerifyBaselineFindAllReferences(t, "1")
}
