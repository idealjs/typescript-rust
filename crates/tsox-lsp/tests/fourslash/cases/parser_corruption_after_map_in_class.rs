use tsox_lsp::fourslash::{self, Session};


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
    let mut s = Session::new_for_test("parserCorruptionAfterMapInClass", content);
    fourslash::go_to_marker(&mut s, "$");
    fourslash::insert(&mut s, "()");
    // TODO: f.VerifyNonSuggestionDiagnostics(t, []*lsproto.Diagnostic{
}
