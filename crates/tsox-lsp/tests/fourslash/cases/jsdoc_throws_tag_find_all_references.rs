use tsox_lsp::fourslash::{self, Session};


#[test]
fn jsdoc_throws_tag_find_all_references() {
    let content = r#"class /**/E extends Error {}
/**
 * @throws {E}
 */
function f() {}"#;
    let mut s = Session::new_for_test("jsdocThrowsTag_findAllReferences", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "")
}
