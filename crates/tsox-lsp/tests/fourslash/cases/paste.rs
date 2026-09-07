use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.Paste"]
#[test]
fn paste() {
    let content = r#"fn(/**/);"#;
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("Paste"); // f.Paste(t, "x,y,z")
    fourslash::verify_current_line_content(&mut s, r#"fn(x, y, z);"#);
}
