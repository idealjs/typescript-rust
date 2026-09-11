use tsox_lsp::fourslash::{self, Session};


#[test]
fn quick_info_display_parts_function_incomplete() {
    let content = r#"/*1*/function /*2*/(param: string) {
}\
/*3*/function /*4*/ {
}\"#;
    let mut s = Session::new_for_test("quickInfoDisplayPartsFunctionIncomplete", content);
    // TODO: f.VerifyBaselineHover(t)
}
