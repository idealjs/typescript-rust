use tsox_lsp::fourslash::{self, Session};


#[test]
fn quick_info_class_keyword() {
    let content = r#"[1].forEach(cla/*1*/ss {});
[1].forEach(cla/*2*/ss OK{});"#;
    let mut s = Session::new_for_test("quickInfoClassKeyword", content);
    fourslash::verify_quick_info_at(&mut s, "1", "(local class) (Anonymous class)", "");
    fourslash::verify_quick_info_at(&mut s, "2", "(local class) OK", "");
}
