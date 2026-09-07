use tsox_lsp::fourslash::{self, Session};

#[test]
fn tsx_quick_info1() {
    let content = r#"//@Filename: file.tsx
var x1 = <di/*1*/v></di/*2*/v>
class MyElement {}
var z = <My/*3*/Element></My/*4*/Element>"#;
    let mut s = Session::new(content);
    fourslash::verify_quick_info_at(&mut s, "1", "any", "");
    fourslash::verify_quick_info_at(&mut s, "2", "any", "");
    fourslash::verify_quick_info_at(&mut s, "3", "class MyElement", "");
    fourslash::verify_quick_info_at(&mut s, "4", "class MyElement", "");
}
