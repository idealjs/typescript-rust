use tsox_lsp::fourslash::Session;


#[test]
fn quick_info_display_parts_interface() {
    let content = r#"interface /*1*/i {
}
var /*2*/iInstance: /*3*/i;"#;
    let _s = Session::new_for_test("quickInfoDisplayPartsInterface", content);
    // TODO: f.VerifyBaselineHover(t)
}
