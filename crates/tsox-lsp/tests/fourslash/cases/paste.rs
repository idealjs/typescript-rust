use tsox_lsp::fourslash::{self, Session};


#[test]
fn paste() {
    let content = r#"fn(/**/);"#;
    let mut s = Session::new_for_test("paste", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.Paste(t, "x,y,z")
    fourslash::verify_current_line_content(&mut s, r#"fn(x, y, z);"#);
}
