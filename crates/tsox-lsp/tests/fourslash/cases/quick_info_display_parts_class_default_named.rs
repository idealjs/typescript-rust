use tsox_lsp::fourslash::{self, Session};


#[test]
fn quick_info_display_parts_class_default_named() {
    let content = r#"/*1*/export /*2*/default /*3*/class /*4*/C /*5*/ {
}"#;
    let mut s = Session::new_for_test("quickInfoDisplayPartsClassDefaultNamed", content);
    // TODO: f.VerifyBaselineHover(t)
}
