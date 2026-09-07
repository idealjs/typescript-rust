use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyQuickInfoAt"]
#[test]
fn string_property_names1() {
    let content = r#"export interface Album {
   "artist": number;
}
var a: Album;
var /**/x = a['artist'];"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "", "var x: number", "")
}
