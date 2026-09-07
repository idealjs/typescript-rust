use tsox_lsp::fourslash::{self, Session};

#[test]
fn constructor_quick_info() {
    let content = r#"class SS<T>{}

var x/*1*/1 = new SS<number>();
var x/*2*/2 = new SS();
var x/*3*/3 = new SS;"#;
    let mut s = Session::new(content);
    fourslash::verify_quick_info_at(&mut s, "1", "var x1: SS<number>", "");
    fourslash::verify_quick_info_at(&mut s, "2", "var x2: SS<unknown>", "");
    fourslash::verify_quick_info_at(&mut s, "3", "var x3: SS<unknown>", "");
}
