use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn references_for_label4() {
    let content = r#"/*1*/label: function foo(label) {
    while (true) {
        /*2*/break /*3*/label;
    }
}"#;
    let mut s = Session::new_for_test("referencesForLabel4", content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1", "2", "3")
}
