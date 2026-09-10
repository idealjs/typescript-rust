use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifySuggestionDiagnostics"]
#[test]
fn jsdoc_deprecated_suggestion15() {
    let content = r#"// @module: esnext
// @filename: /a.ts
export const a = 1;
export const b = 1;
// @filename: /b.ts
export {
    /** @deprecated a is deprecated */
    a
} from "./a";
// @filename: /c.ts
export {
    a
} from "./b";
// @filename: /d.ts
import { [|a|] } from "./c";
[|a|]"#;
    let mut s = Session::new_for_test("jsdocDeprecated_suggestion15", content);
    fourslash::go_to_file(&mut s, "/d.ts");
    fourslash::unsupported("VerifySuggestionDiagnostics"); // f.VerifySuggestionDiagnostics(t, []*lsproto.Diagnostic{
}
