use tsox_lsp::fourslash::{self, Session};


#[test]
fn import_name_code_fix_uri_style_node_core_modules1() {
    let content = r#"// @module: commonjs
// @Filename: /node_modules/@types/node/index.d.ts
declare module "fs" { function writeFile(): void }
declare module "fs/promises" { function writeFile(): Promise<void> }
declare module "node:fs" { export * from "fs"; }
declare module "node:fs/promises" { export * from "fs/promises"; }
// @Filename: /index.ts
writeFile/**/"#;
    let mut s = Session::new_for_test("importNameCodeFix_uriStyleNodeCoreModules1", content);
    // TODO: f.VerifyImportFixModuleSpecifiers(t, "", []string{"fs", "node:fs", "fs/promises", "node:fs/promises"
}
