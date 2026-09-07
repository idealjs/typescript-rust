use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn quick_info_display_parts_iife() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @strictNullChecks: true
var iife = (function foo/*1*/(x, y) { return x })(12);"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "1", "(local function) foo(x: number, y?: undefined): number", "")
}
