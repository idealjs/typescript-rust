use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyQuickInfoIs"]
#[test]
fn js_doc_for_type_alias() {
    let content = r#"/** DOC */
type /**/T = number"#;
    let mut s = Session::new_for_test("jsDocForTypeAlias", content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("VerifyQuickInfoIs"); // f.VerifyQuickInfoIs(t, "type T = number", "DOC")
}
