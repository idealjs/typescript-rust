use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.FormatSelection"]
#[test]
fn format_template_string_on_paste() {
    let content = r#"const x = ` + "`" + `${0}/*0*/abc/*1*/` + "`" + `;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("FormatSelection"); // f.FormatSelection(t, "0", "1")
    fourslash::unsupported("VerifyCurrentFileContent"); // f.VerifyCurrentFileContent(t, "const x = `${0}abc`;")
}
