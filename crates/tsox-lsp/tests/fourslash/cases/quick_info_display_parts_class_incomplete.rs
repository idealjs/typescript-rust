use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineHover"]
#[test]
fn quick_info_display_parts_class_incomplete() {
    let content = r#"/*1*/class /*2*/ {
}"#;
    let mut s = Session::new_for_test("quickInfoDisplayPartsClassIncomplete", content);
    fourslash::unsupported("VerifyBaselineHover"); // f.VerifyBaselineHover(t)
}
