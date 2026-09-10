use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineRename"]
#[test]
fn jsdoc_throws_tag_rename() {
    let content = r#"class /**/E extends Error {}
/**
 * @throws {E}
 */
function f() {}"#;
    let mut s = Session::new_for_test("jsdocThrowsTag_rename", content);
    fourslash::unsupported("VerifyBaselineRename"); // f.VerifyBaselineRename(t, nil /*preferences*/, "")
}
