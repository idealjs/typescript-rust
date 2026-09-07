use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.DeleteAtCaret"]
#[test]
fn remove_interface_extends_clause() {
    let content = r#"interface IFoo<T> { }
interface Array<T> /**/extends IFoo<T> { }"#;
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("DeleteAtCaret"); // f.DeleteAtCaret(t, 15)
}
