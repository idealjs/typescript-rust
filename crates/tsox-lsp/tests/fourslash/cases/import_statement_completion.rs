use tsox_lsp::fourslash::{self, Session};


#[test]
fn import_statement_completion_uses_named_import() {
    let content = r#"// @Filename: a.ts
export interface I {}
// @Filename: 1.ts
import * as u from "./a";
[|import I/*a*/|]"#;
    let mut s = Session::new_for_test("importStatementCompletionUsesNamedImport", content);
    // TODO: preferences := lsutil.NewDefaultUserPreferences()
    // TODO: preferences.IncludeCompletionsForModuleExports = core.TSFalse
    // TODO: preferences.IncludeCompletionsForImportStatements = core.TSTrue
    fourslash::go_to_marker(&mut s, "a");
    // TODO: f.VerifyCompletions(t, "a", &fourslash.CompletionsExpectedList{
}
