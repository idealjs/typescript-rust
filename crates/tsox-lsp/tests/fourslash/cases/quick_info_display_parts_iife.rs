use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn quick_info_display_parts_iife() {
    let content = r#"// @strictNullChecks: true
var iife = (function foo/*1*/(x, y) { return x })(12);"#;
    let mut s = Session::new_for_test("quickInfoDisplayPartsIife", content);
    fourslash::verify_quick_info_at(&mut s, "1", "(local function) foo(x: number, y?: undefined): number", "");
}
