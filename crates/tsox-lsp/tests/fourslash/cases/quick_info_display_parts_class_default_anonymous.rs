use tsox_lsp::fourslash::{self, Session};


#[test]
fn quick_info_display_parts_class_default_anonymous() {
    let content = r#"/*1*/export /*2*/default /*3*/class /*4*/ {
}"#;
    let mut s = Session::new_for_test("quickInfoDisplayPartsClassDefaultAnonymous", content);
    // TODO: f.VerifyBaselineHover(t)
}
