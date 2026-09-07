use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: f.MarkTestAsStradaServer()"]
#[test]
fn auto_import_provider5() {
    let content = r#"// @lib: es5
// @Filename: /home/src/workspaces/project/package.json
{ "dependencies": { "react-hook-form": "*" } }
// @Filename: /home/src/workspaces/project/node_modules/react-hook-form/package.json
{ "types": "dist/index.d.ts" }
// @Filename: /home/src/workspaces/project/node_modules/react-hook-form/dist/index.d.ts
export * from "./useForm";
// @Filename: /home/src/workspaces/project/node_modules/react-hook-form/dist/useForm.d.ts
export declare function useForm(): void;
// @Filename: /home/src/workspaces/project/index.ts
useForm/**/"#;
    let mut s = Session::new(content);
    fourslash::unsupported("Configure"); // f.Configure(t, lsutil.UserPreferences{AutoImportEntrypointDirectorySearch: core.TSTrue})
    // TODO: f.MarkTestAsStradaServer()
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("VerifyImportFixAtPosition"); // f.VerifyImportFixAtPosition(t, []string{
}
