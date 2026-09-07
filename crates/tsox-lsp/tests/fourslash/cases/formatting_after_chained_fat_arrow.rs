use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: f.VerifyCurrentLineContent(t, `"]
#[test]
fn formatting_after_chained_fat_arrow() {
    let content = r#"var x = n => p => {
    while (true) {
        void 0;
    }/**/
};"#;
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("FormatDocument"); // f.FormatDocument(t, "")
    // TODO: f.VerifyCurrentLineContent(t, `
}
