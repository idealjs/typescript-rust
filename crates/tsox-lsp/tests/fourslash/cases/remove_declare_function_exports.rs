use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.DeleteAtCaret"]
#[test]
fn remove_declare_function_exports() {
    let content = r#"declare namespace M {
    function RegExp2(pattern: string): RegExp2;
    export function RegExp2(pattern: string, flags: string): RegExp2;
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("GoToBOF"); // f.GoToBOF(t)
    fourslash::unsupported("DeleteAtCaret"); // f.DeleteAtCaret(t, 8)
}
