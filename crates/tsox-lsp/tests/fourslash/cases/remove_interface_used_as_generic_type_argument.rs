use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.DeleteAtCaret"]
#[test]
fn remove_interface_used_as_generic_type_argument() {
    let content = r#"/**/interface A { a: string; }
interface G<T, U> { }
var v1: G<A, C>;"#;
    let mut s = Session::new_for_test("removeInterfaceUsedAsGenericTypeArgument", content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("DeleteAtCaret"); // f.DeleteAtCaret(t, 26)
}
