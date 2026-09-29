use tsox_lsp::fourslash::Session;


#[test]
fn references_for_label4() {
    let content = r#"/*1*/label: function foo(label) {
    while (true) {
        /*2*/break /*3*/label;
    }
}"#;
    let _s = Session::new_for_test("referencesForLabel4", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2", "3")
}
