use tsox_lsp::fourslash::{self, Session};


#[test]
fn formatting_hex_literal() {
    let content = r#"var x =  0x1,y;"#;
    let mut s = Session::new_for_test("formattingHexLiteral", content);
    fourslash::format_document(&mut s, "");
}
