use tsox_lsp::fourslash::Session;


#[test]
fn go_to_definition_mapped_type() {
    let content = r#"interface I { /*def*/m(): void; };
declare const i: { [K in "m"]: I[K] };
i.[|/*ref*/m|]();"#;
    let _s = Session::new_for_test("goToDefinition_mappedType", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "ref")
}
