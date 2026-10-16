use tsox_lsp::fourslash::Session;


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
    let _s = Session::new_for_test("importTypesDeclarationDiagnosticsNoServerError", content);
    // TODO: f.GoToFileNumber(t, 1)
    // TODO: f.VerifyNonSuggestionDiagnostics(t, nil)
}
