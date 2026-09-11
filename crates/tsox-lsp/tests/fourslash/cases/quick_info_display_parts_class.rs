use tsox_lsp::fourslash::{self, Session};


#[test]
fn quick_info_display_parts_class() {
    let content = r#"class /*1*/c {
}
var /*2*/cInstance = new /*3*/c();
var /*4*/cVal = /*5*/c;"#;
    let mut s = Session::new_for_test("quickInfoDisplayPartsClass", content);
    // TODO: f.VerifyBaselineHover(t)
}
