use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: f.MarkTestAsStradaServer()"]
#[test]
fn auto_import_re_export_from_ambient_module() {
    let content = r#"// @Filename: /home/src/workspaces/project/tsconfig.json
{
  "compilerOptions": {
    "module": "commonjs",
    "types": ["*"],
    "lib": ["es5"]
  }
}
// @Filename: /home/src/workspaces/project/node_modules/@types/node/index.d.ts
declare module "fs" {
  export function accessSync(path: string): void;
}
// @Filename: /home/src/workspaces/project/node_modules/@types/fs-extra/index.d.ts
export * from "fs";
// @Filename: /home/src/workspaces/project/index.ts
access/**/"#;
    let mut s = Session::new(content);
    // TODO: f.MarkTestAsStradaServer()
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyApplyCodeActionFromCompletion"); // f.VerifyApplyCodeActionFromCompletion(t, new(""), &fourslash.ApplyCodeActionFromCompletionOptions{
}
