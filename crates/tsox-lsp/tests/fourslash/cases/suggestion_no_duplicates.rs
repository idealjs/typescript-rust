use tsox_lsp::fourslash::Session;


#[test]
fn suggestion_no_duplicates() {
    let content = r#"// @strict: false
// @Filename: foo.ts
import { f } from [|'m'|]
f
// @Filename: node_modules/m/index.js
module.exports.f = function (x) { return x }"#;
    let _s = Session::new_for_test("suggestionNoDuplicates", content);
    // TODO: f.VerifyNonSuggestionDiagnostics(t, nil)
    // TODO: f.VerifySuggestionDiagnostics(t, []*lsproto.Diagnostic{
}
