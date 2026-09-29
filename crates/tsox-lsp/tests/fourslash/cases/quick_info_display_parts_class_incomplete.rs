use tsox_lsp::fourslash::Session;


#[test]
fn quick_info_display_parts_class_incomplete() {
    let content = r#"/*1*/class /*2*/ {
}"#;
    let _s = Session::new_for_test("quickInfoDisplayPartsClassIncomplete", content);
    // TODO: f.VerifyBaselineHover(t)
}
