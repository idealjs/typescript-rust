use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyOutliningSpans"]
#[test]
fn outlining_for_non_complete_interface_declaration() {
    let content = r#"interface I"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyOutliningSpans"); // f.VerifyOutliningSpans(t)
}
