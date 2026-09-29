use tsox_lsp::fourslash::{self, Session};


#[test]
fn jsdoc_deprecated_suggestion21() {
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
import * as _ from "./c";
_.[|a|]"#;
    let mut s = Session::new_for_test("jsdocDeprecated_suggestion21", content);
    fourslash::go_to_file(&mut s, "/d.ts");
    // TODO: f.VerifySuggestionDiagnostics(t, []*lsproto.Diagnostic{
}
