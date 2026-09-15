use tsox_lsp::fourslash::Session;


#[test]
fn references_for_label3() {
    let content = r#"/*1*/label: while (true) {
    var label = "label";
}"#;
    let _s = Session::new_for_test("referencesForLabel3", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1")
}
