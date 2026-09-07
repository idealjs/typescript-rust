use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyQuickInfoAt"]
#[test]
fn constructor_quick_info() {
    let content = r#"class SS<T>{}

var x/*1*/1 = new SS<number>();
var x/*2*/2 = new SS();
var x/*3*/3 = new SS;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "1", "var x1: SS<number>", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "2", "var x2: SS<unknown>", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "3", "var x3: SS<unknown>", "")
}
