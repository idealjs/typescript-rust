use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyQuickInfoExists"]
#[test]
fn quick_info_of_lablled_for_statement_iterator() {
    let content = r#"label1: for(var /**/i = 0; i < 1; i++) { }"#;
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("VerifyQuickInfoExists"); // f.VerifyQuickInfoExists(t)
}
