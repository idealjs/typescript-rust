use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyQuickInfoAt"]
#[test]
fn quick_info_for_destructuring_shorthand_initializer() {
    let content = r#"let a = '';
let b: string;
({b = /**/a} = {b: 'b'});"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "", "let a: string", "")
}
