use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: f.MarkTestAsStradaServer()"]
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
    fourslash::unsupported("VerifyImportFixAtPosition"); // f.VerifyImportFixAtPosition(t, []string{}, nil /*preferences*/)
    fourslash::unsupported("GoToBOF"); // f.GoToBOF(t)
    fourslash::unsupported("InsertLine"); // f.InsertLine(t, "import { writeFile } from \"node:fs\";")
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("VerifyImportFixAtPosition"); // f.VerifyImportFixAtPosition(t, []string{
}
