use tsox_lsp::fourslash::Session;


#[test]
fn workspace_symbol_multi_project_non_existent_ref() {
    let content = r#"
// @Filename: /home/src/projects/project-a/tsconfig.json
{
  "compilerOptions": { "composite": true },
  "references": [{ "path": "../project-nonexistent" }]
}

// @Filename: /home/src/projects/project-a/index.ts
export const [|myValueA|]: number = 1;

// @Filename: /home/src/projects/project-b/tsconfig.json
{
  "compilerOptions": { "composite": true },
  "references": [{ "path": "../project-a" }]
}

// @Filename: /home/src/projects/project-b/index.ts
export const [|myValueB|]: string = "hello";
"#;
    let _s = Session::new_for_test("workspaceSymbolMultiProjectNonExistentRef", content);
    // TODO: // Verify we can find symbols from both projects with a single pattern
    // TODO: f.VerifyWorkspaceSymbol(t, []*fourslash.VerifyWorkspaceSymbolCase{
}
