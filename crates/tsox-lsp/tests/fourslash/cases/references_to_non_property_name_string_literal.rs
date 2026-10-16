use tsox_lsp::fourslash::Session;


#[test]
fn references_to_non_property_name_string_literal() {
    let content = r#"// @lib: es5
const str: string = "hello/*1*/";"#;
    let _s = Session::new_for_test("referencesToNonPropertyNameStringLiteral", content);
    // TODO: f.MarkTestAsStradaServer()
    // TODO: f.VerifyBaselineFindAllReferences(t, "1")
}
