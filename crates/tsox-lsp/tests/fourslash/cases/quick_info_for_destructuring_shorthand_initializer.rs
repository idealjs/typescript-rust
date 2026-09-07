use tsox_lsp::fourslash::{self, Session};

#[test]
fn quick_info_for_destructuring_shorthand_initializer() {
    let content = r#"let a = '';
let b: string;
({b = /**/a} = {b: 'b'});"#;
    let mut s = Session::new(content);
    fourslash::verify_quick_info_at(&mut s, "", "let a: string", "");
}
