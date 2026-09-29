use tsox_lsp::fourslash::{self, Session};


#[test]
fn formatting_after_chained_fat_arrow() {
    let content = r#"var x = n => p => {
    while (true) {
        void 0;
    }/**/
};"#;
    let mut s = Session::new_for_test("formattingAfterChainedFatArrow", content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::format_document(&mut s, "");
    // TODO: f.VerifyCurrentLineContent(t, `
}
