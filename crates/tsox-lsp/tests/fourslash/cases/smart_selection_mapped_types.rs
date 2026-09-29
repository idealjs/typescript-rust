use tsox_lsp::fourslash::Session;


#[test]
fn smart_selection_mapped_types() {
    let content = r#"type M = { /*1*/-re/*2*/adonly /*3*/[K in ke/*4*/yof any]/*5*/-/*6*/?: any };"#;
    let _s = Session::new_for_test("smartSelection_mappedTypes", content);
    // TODO: f.VerifyBaselineSelectionRanges(t)
}
