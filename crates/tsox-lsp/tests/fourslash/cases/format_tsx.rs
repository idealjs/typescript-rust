use tsox_lsp::fourslash::{self, Session};


#[test]
fn format_tsx() {
    let content = r#"// @Filename: foo.tsx
<div><p>'</p><p>{function(){return 1;}]}</p></div>"#;
    let mut s = Session::new_for_test("formatTsx", content);
    fourslash::format_document(&mut s, "");
    fourslash::verify_current_file_content(&mut s, r#"<div><p>'</p><p>{function() { return 1; }]}</p></div>"#);
}
