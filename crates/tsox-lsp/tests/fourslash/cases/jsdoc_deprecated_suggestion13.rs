use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifySuggestionDiagnostics"]
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
    let mut s = Session::new(content);
    fourslash::go_to_file(&mut s, "foo.ts");
    fourslash::unsupported("VerifySuggestionDiagnostics"); // f.VerifySuggestionDiagnostics(t, []*lsproto.Diagnostic{
}
