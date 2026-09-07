use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.DeleteAtCaret"]
#[test]
fn remove_var_from_module_with_reopened_enums() {
    let content = r#"namespace A {
    /**/var o;
}
enum A {
}
enum A {
}
namespace A {
    var p;
}"#;
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("DeleteAtCaret"); // f.DeleteAtCaret(t, 6)
}
