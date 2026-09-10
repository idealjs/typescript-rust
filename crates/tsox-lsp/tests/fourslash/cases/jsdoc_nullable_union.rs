use tsox_lsp::fourslash::{self, Session};


#[test]
fn jsdoc_nullable_union() {
    let content = r#"// @allowNonTsExtensions: true
// @checkJs: true
// @Filename: Foo.js
/**
 * @param {never | {x: string}} p1
 * @param {undefined | {y: number}} p2
 * @param {null | {z: boolean}} p3
 * @returns {void} nothing
 */
function f(p1, p2, p3) {
    p1./*1*/;
    p2./*2*/;
    p3./*3*/;
}"#;
    let mut s = Session::new_for_test("jsdocNullableUnion", content);
    fourslash::verify_completions_exact_at(&mut s, Some("1"), &["x"]);
    fourslash::verify_completions_exact_at(&mut s, Some("2"), &["y"]);
    fourslash::verify_completions_exact_at(&mut s, Some("3"), &["z"]);
}
