use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.DeleteAtCaret"]
#[test]
fn remove_declare_keyword() {
    let content = r#"/**/declare var y;
var x = new y;"#;
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("DeleteAtCaret"); // f.DeleteAtCaret(t, 7)
}
