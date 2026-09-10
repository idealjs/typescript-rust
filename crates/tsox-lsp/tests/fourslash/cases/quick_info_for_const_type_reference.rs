use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyNotQuickInfoExists"]
#[test]
fn quick_info_for_const_type_reference() {
    let content = r#""" as /**/const;"#;
    let mut s = Session::new_for_test("quickInfoForConstTypeReference", content);
    fourslash::unsupported("VerifyNotQuickInfoExists"); // f.VerifyNotQuickInfoExists(t)
}
