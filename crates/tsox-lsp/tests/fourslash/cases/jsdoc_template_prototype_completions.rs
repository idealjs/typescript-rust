use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn jsdoc_template_prototype_completions() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @checkJs: true
// @filename: index.js
https://github.com/microsoft/TypeScript/issues/11492
/** @constructor */
function Foo() {}
/**
 * @template T
 * @param {T} bar
 * @returns {T}
 */
Foo.prototype.foo = function (bar) {};
new Foo().foo({ id: 1234 })./**/"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
