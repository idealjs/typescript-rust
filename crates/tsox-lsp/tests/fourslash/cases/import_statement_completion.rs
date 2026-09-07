use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: preferences := lsutil.NewDefaultUserPreferences()"]
#[test]
fn import_statement_completion_uses_named_import() {
    let content = r#"// @Filename: a.ts
export interface I {}
// @Filename: 1.ts
import * as u from "./a";
[|import I/*a*/|]"#;
    let mut s = Session::new(content);
    // TODO: preferences := lsutil.NewDefaultUserPreferences()
    // TODO: preferences.IncludeCompletionsForModuleExports = core.TSFalse
    // TODO: preferences.IncludeCompletionsForImportStatements = core.TSTrue
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "a", &fourslash.CompletionsExpectedList{
}
