use tsox_lsp::fourslash::{self, Session};


#[test]
fn references_for_label6() {
    let content = r#"/*1*/labela: while (true) {
/*2*/labelb:     while (false) { /*3*/break /*4*/labelb; }
            break labelc;
}"#;
    let mut s = Session::new_for_test("referencesForLabel6", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2", "3", "4")
}
