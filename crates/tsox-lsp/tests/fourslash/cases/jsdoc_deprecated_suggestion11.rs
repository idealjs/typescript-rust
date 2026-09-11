use tsox_lsp::fourslash::{self, Session};


#[test]
fn jsdoc_deprecated_suggestion11() {
    let content = r#"// @filename: /foo.ts
/** @deprecated */
export function foo() {}
// @filename: /test.ts
import { [|foo|] } from "./foo";
[|foo|];"#;
    let mut s = Session::new_for_test("jsdocDeprecated_suggestion11", content);
    fourslash::go_to_file(&mut s, "/test.ts");
    // TODO: f.VerifySuggestionDiagnostics(t, []*lsproto.Diagnostic{
}
