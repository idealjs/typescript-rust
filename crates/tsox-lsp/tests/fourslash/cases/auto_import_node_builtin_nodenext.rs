use tsox_lsp::fourslash::{self, Session};


#[test]
fn auto_import_node_builtin_nodenext() {
    let content = r#"// @Filename: /tsconfig.json
{ "compilerOptions": { "module": "nodenext", "types": ["node"] } }
// @Filename: /package.json
{ "type": "module" }
// @Filename: /node_modules/@types/node/package.json
{ "name": "@types/node", "version": "22.0.0" }
// @Filename: /node_modules/@types/node/index.d.ts
declare module "fs" {
    export function existsSync(path: string): boolean;
    export function mkdirSync(path: string, options?: { recursive?: boolean }): void;
}
declare module "node:fs" { export * from "fs"; }
// @Filename: /index.ts
existsSync/**/"#;
    let mut s = Session::new_for_test("autoImportNodeBuiltinNodenext", content);
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
