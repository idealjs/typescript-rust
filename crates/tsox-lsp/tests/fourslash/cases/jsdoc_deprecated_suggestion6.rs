use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifySuggestionDiagnostics"]
#[test]
fn jsdoc_deprecated_suggestion6() {
    let content = r#"// @Filename: a.tsx
/** @deprecated */
type Props = {}
/** @deprecated */
const Component = (props: [|Props|]) => props && <div />;
<[|Component|] old="old" new="new" />
/** @deprecated */
type Options = {}
/** @deprecated */
const deprecatedFunction = (options: [|Options|]) => { options }
[|deprecatedFunction|]({});"#;
    let mut s = Session::new_for_test("jsdocDeprecated_suggestion6", content);
    fourslash::go_to_file(&mut s, "a.tsx");
    fourslash::unsupported("VerifySuggestionDiagnostics"); // f.VerifySuggestionDiagnostics(t, []*lsproto.Diagnostic{
}
