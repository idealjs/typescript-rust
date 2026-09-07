use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn quick_info_function_keyword() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"[1].forEach(fu/*1*/nction() {});
[1].map(x =/*2*/> x + 1);"#;
    let mut s = Session::new(content);
    fourslash::verify_quick_info_at(&mut s, "1", "(local function)(): void", "");
    fourslash::verify_quick_info_at(&mut s, "2", "function(x: number): number", "");
}
