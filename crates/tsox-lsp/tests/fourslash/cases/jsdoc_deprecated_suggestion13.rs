use tsox_lsp::fourslash::{self, Session};


#[test]
fn jsdoc_deprecated_suggestion13() {
    let content = r#"// @filename: foo.ts
/**
 * @deprecated
 */
function foo() {};

class Foo {
    constructor(fn: () => void) {
        fn();
    }
}
new Foo([|foo|]);"#;
    let mut s = Session::new_for_test("jsdocDeprecated_suggestion13", content);
    fourslash::go_to_file(&mut s, "foo.ts");
    // TODO: f.VerifySuggestionDiagnostics(t, []*lsproto.Diagnostic{
}
