use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: allOpenProjects := lsutil.NewDefaultUserPreferences()"]
#[test]
fn workspace_symbol_current_project() {
    let content = r#"
// @Filename: /home/projects/a/tsconfig.json
{}

// @Filename: /home/projects/a/index.ts
export function [|fromA|]() {}

// @Filename: /home/projects/b/tsconfig.json
{}

// @Filename: /home/projects/b/index.ts
export function [|fromB|]() {}
"#;
    let mut s = Session::new_for_test("workspaceSymbolCurrentProject", content);
    fourslash::go_to_file(&mut s, "/home/projects/a/index.ts");
    // TODO: allOpenProjects := lsutil.NewDefaultUserPreferences()
    // TODO: currentProject := lsutil.NewDefaultUserPreferences()
    // TODO: currentProject.WorkspaceSymbolsScope = lsutil.WorkspaceSymbolsScopeCurrentProject
    fourslash::unsupported("VerifyWorkspaceSymbol"); // f.VerifyWorkspaceSymbol(t, []*fourslash.VerifyWorkspaceSymbolCase{
}
