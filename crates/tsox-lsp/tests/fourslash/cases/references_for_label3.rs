use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn references_for_label3() {
    let content = r#"/*1*/label: while (true) {
    var label = "label";
}"#;
    let mut s = Session::new_for_test("referencesForLabel3", content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1")
}
