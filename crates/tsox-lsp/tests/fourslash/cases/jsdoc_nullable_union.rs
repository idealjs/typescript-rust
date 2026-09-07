use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
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
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "3", &fourslash.CompletionsExpectedList{
}
