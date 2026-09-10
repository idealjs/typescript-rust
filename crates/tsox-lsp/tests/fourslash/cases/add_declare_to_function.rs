use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.DeleteAtCaret"]
#[test]
fn add_declare_to_function() {
    let content = r#"/*1*/function parseInt(s/*2*/:string):number;"#;
    let mut s = Session::new_for_test("addDeclareToFunction", content);
    fourslash::go_to_marker(&mut s, "2");
    fourslash::unsupported("DeleteAtCaret"); // f.DeleteAtCaret(t, 7)
    fourslash::go_to_marker(&mut s, "1");
    fourslash::insert(&mut s, "declare ");
}
