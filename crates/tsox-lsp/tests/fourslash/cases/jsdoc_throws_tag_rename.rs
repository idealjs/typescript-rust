use tsox_lsp::fourslash::Session;


#[test]
fn jsdoc_throws_tag_rename() {
    let content = r#"class /**/E extends Error {}
/**
 * @throws {E}
 */
function f() {}"#;
    let _s = Session::new_for_test("jsdocThrowsTag_rename", content);
    // TODO: f.VerifyBaselineRename(t, nil /*preferences*/, "")
}
