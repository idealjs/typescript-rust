use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyQuickInfoAt"]
#[test]
fn quick_info_from_empty_block_comment() {
    let content = r#"/**/
class Foo {
}
var f/*A*/ff = new Foo();"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "A", "var fff: Foo", "")
}
