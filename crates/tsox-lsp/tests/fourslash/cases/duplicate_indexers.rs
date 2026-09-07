use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyQuickInfoAt"]
#[test]
fn duplicate_indexers() {
    let content = r#"interface I {
    [x: number]: string;
    [x: number]: number;
}
var i: I;
var /**/r = i[1];"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "", "var r: string", "")
}
