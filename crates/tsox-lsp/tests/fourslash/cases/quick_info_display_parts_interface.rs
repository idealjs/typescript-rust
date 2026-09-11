use tsox_lsp::fourslash::{self, Session};


#[test]
fn quick_info_display_parts_interface() {
    let content = r#"interface /*1*/i {
}
var /*2*/iInstance: /*3*/i;"#;
    let mut s = Session::new_for_test("quickInfoDisplayPartsInterface", content);
    // TODO: f.VerifyBaselineHover(t)
}
