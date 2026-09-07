use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn references_for_label6() {
    let content = r#"/*1*/labela: while (true) {
/*2*/labelb:     while (false) { /*3*/break /*4*/labelb; }
            break labelc;
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1", "2", "3", "4")
}
