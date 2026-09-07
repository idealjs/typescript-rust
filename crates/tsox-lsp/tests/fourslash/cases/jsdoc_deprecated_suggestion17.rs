use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifySuggestionDiagnostics"]
#[test]
fn jsdoc_deprecated_suggestion17() {
    let content = r#"// @filename: foo.ts
interface Foo {
    /** @deprecated */
    [k: string]: any;
    /** @deprecated please use ` + "`" + `.y` + "`" + ` instead  */
    x: number;
    y: number;
}
function f(foo: Foo) {
    foo.[|x|];
    foo.[|y|];
    foo.[|z|];
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifySuggestionDiagnostics"); // f.VerifySuggestionDiagnostics(t, []*lsproto.Diagnostic{
}
