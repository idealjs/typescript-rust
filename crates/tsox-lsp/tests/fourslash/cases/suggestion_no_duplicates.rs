use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyNonSuggestionDiagnostics"]
#[test]
fn suggestion_no_duplicates() {
    let content = r#"// @strict: false
// @Filename: foo.ts
import { f } from [|'m'|]
f
// @Filename: node_modules/m/index.js
module.exports.f = function (x) { return x }"#;
    let mut s = Session::new_for_test("suggestionNoDuplicates", content);
    fourslash::unsupported("VerifyNonSuggestionDiagnostics"); // f.VerifyNonSuggestionDiagnostics(t, nil)
    fourslash::unsupported("VerifySuggestionDiagnostics"); // f.VerifySuggestionDiagnostics(t, []*lsproto.Diagnostic{
}
