use tsox_lsp::fourslash::{self, Session};

#[test]
fn string_property_names2() {
    let content = r#"export interface Album<T> {
   "artist": T;
}
var a: Album<number>;
var /**/x = a['artist']; "#;
    let mut s = Session::new(content);
    fourslash::verify_quick_info_at(&mut s, "", "var x: number", "");
}
