use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyQuickInfoAt"]
#[test]
fn string_property_names2() {
    let content = r#"export interface Album<T> {
   "artist": T;
}
var a: Album<number>;
var /**/x = a['artist']; "#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "", "var x: number", "")
}
