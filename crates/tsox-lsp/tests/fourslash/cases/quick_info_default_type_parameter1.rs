use tsox_lsp::fourslash::{self, Session};


#[test]
fn quick_info_default_type_parameter1() {
    let content = r#"type /*1*/X</*2*/T = string> = T"#;
    let mut s = Session::new_for_test("quickInfoDefaultTypeParameter1", content);
    fourslash::verify_quick_info_at(&mut s, "1", "type X<T = string> = T", "");
    fourslash::verify_quick_info_at(&mut s, "2", "(type parameter) T in type X<T = string>", "");
}
