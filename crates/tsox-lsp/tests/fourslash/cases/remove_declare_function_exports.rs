use tsox_lsp::fourslash::{self, Session};


#[test]
fn remove_declare_function_exports() {
    let content = r#"declare namespace M {
    function RegExp2(pattern: string): RegExp2;
    export function RegExp2(pattern: string, flags: string): RegExp2;
}"#;
    let mut s = Session::new_for_test("removeDeclareFunctionExports", content);
    fourslash::go_to_bof(&mut s, );
    // TODO: f.DeleteAtCaret(t, 8)
}
