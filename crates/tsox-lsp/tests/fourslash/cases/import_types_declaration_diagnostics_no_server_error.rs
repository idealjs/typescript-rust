use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.GoToFileNumber"]
#[test]
fn import_types_declaration_diagnostics_no_server_error() {
    let content = r#"// @declaration: true
// @Filename: node_modules/foo/index.d.ts
export function f(): I;
export interface I {
  x: number;
}
// @Filename: a.ts
import { f } from "foo";
export const x = f();"#;
    let mut s = Session::new(content);
    fourslash::unsupported("GoToFileNumber"); // f.GoToFileNumber(t, 1)
    fourslash::unsupported("VerifyNonSuggestionDiagnostics"); // f.VerifyNonSuggestionDiagnostics(t, nil)
}
