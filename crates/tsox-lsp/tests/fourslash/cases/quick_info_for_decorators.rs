use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyQuickInfoAt"]
#[test]
fn quick_info_for_decorators() {
    let content = r#"@/*1*/decorator
class C {
}
/** decorator documentation*/
var decorator = t=> t;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "1", "var decorator: (t: any) => any", "decorator documentation")
}
