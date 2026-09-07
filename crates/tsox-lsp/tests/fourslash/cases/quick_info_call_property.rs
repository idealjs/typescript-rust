use tsox_lsp::fourslash::{self, Session};

#[test]
fn quick_info_call_property() {
    let content = r#"interface I {
    /** Doc */
    m: () => void;
}
function f(x: I): void {
    x./**/m();
}"#;
    let mut s = Session::new(content);
    fourslash::verify_quick_info_at(&mut s, "", "(property) I.m: () => void", "Doc");
}
