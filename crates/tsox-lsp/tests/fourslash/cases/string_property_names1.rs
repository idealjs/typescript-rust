use tsox_lsp::fourslash::{self, Session};

#[test]
fn string_property_names1() {
    let content = r#"export interface Album {
   "artist": number;
}
var a: Album;
var /**/x = a['artist'];"#;
    let mut s = Session::new(content);
    fourslash::verify_quick_info_at(&mut s, "", "var x: number", "");
}
