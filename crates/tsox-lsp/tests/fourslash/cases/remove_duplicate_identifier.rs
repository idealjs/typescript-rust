use tsox_lsp::fourslash::{self, Session};


#[test]
fn remove_duplicate_identifier() {
    let content = r#"class foo{}
function foo() { return null; }"#;
    let mut s = Session::new_for_test("removeDuplicateIdentifier", content);
    fourslash::go_to_bof(&mut s, );
    // TODO: f.DeleteAtCaret(t, 11)
}
