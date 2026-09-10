use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifySuggestionDiagnostics"]
#[test]
fn jsdoc_deprecated_suggestion16() {
    let content = r#"// @module: esnext
// @filename: /a.ts
const a = 1;
const b = 1;
export { a, /** @deprecated b is deprecated */ b }
// @filename: /b.ts
import { [|b|] } from "./a";
[|b|]"#;
    let mut s = Session::new_for_test("jsdocDeprecated_suggestion16", content);
    fourslash::go_to_file(&mut s, "/b.ts");
    fourslash::unsupported("VerifySuggestionDiagnostics"); // f.VerifySuggestionDiagnostics(t, []*lsproto.Diagnostic{
}
