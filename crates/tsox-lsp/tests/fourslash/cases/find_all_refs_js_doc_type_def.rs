use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn find_all_refs_js_doc_type_def() {
    let content = r#"/** @typedef {Object} /*0*/T */
function foo() {}"#;
    let mut s = Session::new_for_test("findAllRefsJsDocTypeDef", content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "0")
}
