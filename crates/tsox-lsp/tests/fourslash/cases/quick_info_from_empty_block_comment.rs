use tsox_lsp::fourslash::{self, Session};


#[test]
fn quick_info_from_empty_block_comment() {
    let content = r#"/**/
class Foo {
}
var f/*A*/ff = new Foo();"#;
    let mut s = Session::new_for_test("quickInfoFromEmptyBlockComment", content);
    fourslash::verify_quick_info_at(&mut s, "A", "var fff: Foo", "");
}
