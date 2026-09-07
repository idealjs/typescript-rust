use tsox_lsp::fourslash::{self, Session};

#[test]
fn index_signature_without_annotation() {
    let content = r#"interface B {
    1: any;
}
interface C {
    [s]: any;
}
interface D extends B, C /**/ {
}"#;
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::insert(&mut s, " ");
}
