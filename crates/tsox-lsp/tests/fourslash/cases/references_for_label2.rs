use tsox_lsp::fourslash::{self, Session};


#[test]
fn references_for_label2() {
    let content = r#"var label = "label";
while (true) {
    if (false) break /**/label;
    if (true) continue label;
}"#;
    let mut s = Session::new_for_test("referencesForLabel2", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "")
}
