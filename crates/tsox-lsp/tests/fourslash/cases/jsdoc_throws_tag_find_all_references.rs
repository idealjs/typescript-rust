use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn jsdoc_throws_tag_find_all_references() {
    let content = r#"class /**/E extends Error {}
/**
 * @throws {E}
 */
function f() {}"#;
    let mut s = Session::new_for_test("jsdocThrowsTag_findAllReferences", content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "")
}
