use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyNonSuggestionDiagnostics"]
#[test]
fn parser_corruption_after_map_in_class() {
    let content = r#"// @target: esnext
// @lib: es2015
// @strict: true
class C {
    map = new Set<[|string, number|]>/*$*/

    foo() {

    }
}"#;
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "$");
    fourslash::insert(&mut s, "()");
    fourslash::unsupported("VerifyNonSuggestionDiagnostics"); // f.VerifyNonSuggestionDiagnostics(t, []*lsproto.Diagnostic{
}
