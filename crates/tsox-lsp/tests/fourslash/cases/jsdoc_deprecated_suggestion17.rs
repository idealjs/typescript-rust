use tsox_lsp::fourslash::{self, Session};


#[test]
fn jsdoc_deprecated_suggestion17() {
    let content = r#"// @filename: foo.ts
interface Foo {
    /** @deprecated */
    [k: string]: any;
    /** @deprecated please use `.y` instead  */
    x: number;
    y: number;
}
function f(foo: Foo) {
    foo.[|x|];
    foo.[|y|];
    foo.[|z|];
}"#;
    let mut s = Session::new_for_test("jsdocDeprecated_suggestion17", content);
    // TODO: f.VerifySuggestionDiagnostics(t, []*lsproto.Diagnostic{
}
