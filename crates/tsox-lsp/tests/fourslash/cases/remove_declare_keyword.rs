use tsox_lsp::fourslash::{self, Session};


#[test]
fn remove_declare_keyword() {
    let content = r#"/**/declare var y;
var x = new y;"#;
    let mut s = Session::new_for_test("removeDeclareKeyword", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.DeleteAtCaret(t, 7)
}
