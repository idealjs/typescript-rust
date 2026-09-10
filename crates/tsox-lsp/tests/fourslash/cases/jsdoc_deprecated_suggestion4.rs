use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifySuggestionDiagnostics"]
#[test]
fn jsdoc_deprecated_suggestion4() {
    let content = r#"// @jsx: preserve
// @filename: a.tsx
interface Props {
    /** @deprecated */
    x: number
    y: number
}
function A(props: Props) {
    return <div>{props.y}</div>
}
function B() {
    return <A [|x|]={1} y={1} />
}"#;
    let mut s = Session::new_for_test("jsdocDeprecated_suggestion4", content);
    fourslash::go_to_file(&mut s, "a.tsx");
    fourslash::unsupported("VerifySuggestionDiagnostics"); // f.VerifySuggestionDiagnostics(t, []*lsproto.Diagnostic{
}
