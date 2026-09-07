use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifySuggestionDiagnostics"]
#[test]
fn jsdoc_deprecated_suggestion11() {
    let content = r#"// @filename: /foo.ts
/** @deprecated */
export function foo() {}
// @filename: /test.ts
import { [|foo|] } from "./foo";
[|foo|];"#;
    let mut s = Session::new(content);
    fourslash::go_to_file(&mut s, "/test.ts");
    fourslash::unsupported("VerifySuggestionDiagnostics"); // f.VerifySuggestionDiagnostics(t, []*lsproto.Diagnostic{
}
