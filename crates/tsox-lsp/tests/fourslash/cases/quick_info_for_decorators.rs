use tsox_lsp::fourslash::{self, Session};


#[test]
fn quick_info_for_decorators() {
    let content = r#"@/*1*/decorator
class C {
}
/** decorator documentation*/
var decorator = t=> t;"#;
    let mut s = Session::new_for_test("quickInfoForDecorators", content);
    fourslash::verify_quick_info_at(&mut s, "1", "var decorator: (t: any) => any", "decorator documentation");
}
