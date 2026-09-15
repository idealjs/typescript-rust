use tsox_lsp::fourslash::Session;


#[test]
fn references_for_label2() {
    let content = r#"var label = "label";
while (true) {
    if (false) break /**/label;
    if (true) continue label;
}"#;
    let _s = Session::new_for_test("referencesForLabel2", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "")
}
