use tsox_lsp::fourslash::{self, Session};


#[test]
fn duplicate_type_parameters() {
    let content = r#"class A<B, /**/B>  { }"#;
    let mut s = Session::new_for_test("duplicateTypeParameters", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyQuickInfoExists(t)
}
