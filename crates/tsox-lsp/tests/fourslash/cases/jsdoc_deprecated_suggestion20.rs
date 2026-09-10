use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifySuggestionDiagnostics"]
#[test]
fn jsdoc_deprecated_suggestion20() {
    let content = r#"// @module: esnext
// @filename: /a.ts
export default function a() {}
// @filename: /b.ts
import _a from "./a";
export {
	/** @deprecated a is deprecated */
	_a as a,
};
/** @deprecated b is deprecated */
export const b = (): void => {};
// @filename: /c.ts
import * as _ from "./b";

_.[|a|]()
_.[|b|]()"#;
    let mut s = Session::new_for_test("jsdocDeprecated_suggestion20", content);
    fourslash::go_to_file(&mut s, "/c.ts");
    fourslash::unsupported("VerifySuggestionDiagnostics"); // f.VerifySuggestionDiagnostics(t, []*lsproto.Diagnostic{
}
