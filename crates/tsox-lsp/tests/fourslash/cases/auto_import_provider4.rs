use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: f.MarkTestAsStradaServer()"]
#[test]
fn auto_import_provider4() {
    let content = r#"// @Filename: /home/src/workspaces/project/a/package.json
{ "dependencies": { "b": "*" } }
// @Filename: /home/src/workspaces/project/a/tsconfig.json
{ "compilerOptions": { "lib": ["es5"], "module": "commonjs", "target": "esnext" }, "references": [{ "path": "../b" }] }
// @Filename: /home/src/workspaces/project/a/index.ts
new Shape/**/
// @Filename: /home/src/workspaces/project/b/package.json
{ "types": "out/index.d.ts" }
// @Filename: /home/src/workspaces/project/b/tsconfig.json
{ "compilerOptions": { "lib": ["es5"], "outDir": "out", "composite": true } }
// @Filename: /home/src/workspaces/project/b/index.ts
export class Shape {}
// @link: /home/src/workspaces/project/b -> /home/src/workspaces/project/a/node_modules/b"#;
    let mut s = Session::new_for_test("autoImportProvider4", content);
    // TODO: f.MarkTestAsStradaServer()
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("VerifyImportFixAtPosition"); // f.VerifyImportFixAtPosition(t, []string{
}
