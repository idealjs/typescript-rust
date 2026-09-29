use tsox_lsp::fourslash::Session;


#[test]
fn outlining_for_non_complete_interface_declaration() {
    let content = r#"interface I"#;
    let _s = Session::new_for_test("outliningForNonCompleteInterfaceDeclaration", content);
    // TODO: f.VerifyOutliningSpans(t)
}
