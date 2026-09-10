use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: f.MarkTestAsStradaServer()"]
#[test]
fn jsdoc_param_tag_special_keywords() {
    let content = r#"// @lib: es5
// @allowNonTsExtensions: true
// @Filename: test.js
/**
 * @param {string} type
 */
function test(type) {
    type./**/
}"#;
    let mut s = Session::new_for_test("jsdocParamTagSpecialKeywords", content);
    // TODO: f.MarkTestAsStradaServer()
    fourslash::verify_completions_include_exclude_at(&mut s, Some(""), &["charAt"], &[]);
}
