use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn completions_import_uri_style_node_core_modules2() {
    let content = r#"// @lib: es5
// @module: commonjs
// @Filename: /node_modules/@types/node/index.d.ts
declare module "fs" { function writeFile(): void }
declare module "fs/promises" { function writeFile(): Promise<void> }
declare module "node:fs" { export * from "fs"; }
declare module "node:fs/promises" { export * from "fs/promises"; }
// @Filename: /other.ts
import "node:fs/promises";
// @Filename: /index.ts
write/**/"#;
    let mut s = Session::new_for_test("completionsImport_uriStyleNodeCoreModules2", content);
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
