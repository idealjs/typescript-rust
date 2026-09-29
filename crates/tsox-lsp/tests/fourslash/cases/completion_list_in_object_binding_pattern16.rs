use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_in_object_binding_pattern16() {
    let content = r#"// @allowJs: true
// @checkJs: true
// @filename: a.js
/**
 * @typedef Foo
 * @property {number} a
 * @property {string} b
 */

/**
 * @param {Foo} options
 */
function f({ /**/ }) {}"#;
    let mut s = Session::new_for_test("completionListInObjectBindingPattern16", content);
    fourslash::verify_completions_exact_at(&mut s, Some(""), &["a", "b"]);
}
