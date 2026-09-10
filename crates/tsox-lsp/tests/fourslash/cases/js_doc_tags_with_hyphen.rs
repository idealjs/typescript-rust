use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn js_doc_tags_with_hyphen() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @allowJs: true
// @Filename: dummy.js
/**
 * @typedef Product
 * @property {string} title
 * @property {boolean} h/*1*/igh-top some-comments
 */

/**
 * @type {Pro/*2*/duct}
 */
const product = {
    /*3*/
}"#;
    let mut s = Session::new_for_test("jsDocTagsWithHyphen", content);
    fourslash::verify_quick_info_at(&mut s, "1", "(property) high-top: boolean", "some-comments");
    fourslash::verify_quick_info_at(&mut s, "2", "type Product = {\n    title: string;\n    \"high-top\": boolean;\n}", "");
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, []string{"3"}, &fourslash.CompletionsExpectedList{
}
