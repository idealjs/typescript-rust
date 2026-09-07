use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn quick_info_function_keyword() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"[1].forEach(fu/*1*/nction() {});
[1].map(x =/*2*/> x + 1);"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "1", "(local function)(): void", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "2", "function(x: number): number", "")
}
