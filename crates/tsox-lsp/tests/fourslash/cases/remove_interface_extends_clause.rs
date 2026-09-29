use tsox_lsp::fourslash::{self, Session};


#[test]
fn remove_interface_extends_clause() {
    let content = r#"interface IFoo<T> { }
interface Array<T> /**/extends IFoo<T> { }"#;
    let mut s = Session::new_for_test("removeInterfaceExtendsClause", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.DeleteAtCaret(t, 15)
}
