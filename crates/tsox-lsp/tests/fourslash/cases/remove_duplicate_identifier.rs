use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.DeleteAtCaret"]
#[test]
fn remove_duplicate_identifier() {
    let content = r#"class foo{}
function foo() { return null; }"#;
    let mut s = Session::new(content);
    fourslash::unsupported("GoToBOF"); // f.GoToBOF(t)
    fourslash::unsupported("DeleteAtCaret"); // f.DeleteAtCaret(t, 11)
}
