use tsox_lsp::fourslash::{self, Session};


#[test]
fn auto_import_package_json_filter_existing_import3() {
    let content = r#"// @Filename: /home/src/workspaces/project/tsconfig.json
{ "compilerOptions": { "lib": ["es5"], "module": "preserve", "types": ["*"] } }
// @Filename: /home/src/workspaces/project/node_modules/@types/node/index.d.ts
declare module "node:fs" {
    export function readFile(): void;
    export function writeFile(): void;
}
// @Filename: /home/src/workspaces/project/package.json
{}
// @Filename: /home/src/workspaces/project/index.ts
readFile/**/"#;
    let mut s = Session::new_for_test("autoImportPackageJsonFilterExistingImport3", content);
    // TODO: f.MarkTestAsStradaServer()
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyImportFixAtPosition(t, []string{}, nil /*preferences*/)
    // TODO: f.GoToBOF(t)
    // TODO: f.InsertLine(t, "import { writeFile } from \"node:fs\";")
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyImportFixAtPosition(t, []string{
}
