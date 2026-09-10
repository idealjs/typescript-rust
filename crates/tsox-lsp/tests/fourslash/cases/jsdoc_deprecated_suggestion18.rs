use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifySuggestionDiagnostics"]
#[test]
fn jsdoc_deprecated_suggestion18() {
    let content = r#"// @jsx: preserve
// @filename: foo.tsx
interface Props {
    /** @deprecated  */
    x: number;
    y: number;
}
function A(props: Props) {
    return <div>{props.y}</div>
}
function B() {
    return <A [|x|]={1} [|y|]={1} />
}"#;
    let mut s = Session::new_for_test("jsdocDeprecated_suggestion18", content);
    fourslash::unsupported("VerifySuggestionDiagnostics"); // f.VerifySuggestionDiagnostics(t, []*lsproto.Diagnostic{
}
