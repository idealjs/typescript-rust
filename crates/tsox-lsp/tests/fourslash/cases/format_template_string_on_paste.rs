use tsox_lsp::fourslash::{self, Session};


#[test]
fn format_template_string_on_paste() {
    let content = r#"const x = `${0}/*0*/abc/*1*/`;"#;
    let mut s = Session::new_for_test("formatTemplateStringOnPaste", content);
    // TODO: f.FormatSelection(t, "0", "1")
    // TODO: f.VerifyCurrentFileContent(t, "const x = `${0}abc`;")
}
