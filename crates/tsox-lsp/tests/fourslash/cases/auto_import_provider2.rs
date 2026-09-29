use tsox_lsp::fourslash::{self, Session};


#[test]
fn auto_import_provider2() {
    let content = r#"// @Filename: /home/src/workspaces/project/node_modules/direct-dependency/package.json
{ "name": "direct-dependency", "dependencies": { "indirect-dependency": "*" } }
// @Filename: /home/src/workspaces/project/node_modules/direct-dependency/index.d.ts
import "indirect-dependency";
export declare class DirectDependency {}
// @Filename: /home/src/workspaces/project/node_modules/indirect-dependency/package.json
{ "name": "indirect-dependency" }
// @Filename: /home/src/workspaces/project/node_modules/indirect-dependency/index.d.ts
export declare class IndirectDependency
// @Filename: /home/src/workspaces/project/tsconfig.json
{ "compilerOptions": { "lib": ["es5"] } }
// @Filename: /home/src/workspaces/project/package.json
{ "dependencies": { "direct-dependency": "*" } }
// @Filename: /home/src/workspaces/project/index.ts
IndirectDependency/**/"#;
    let mut s = Session::new_for_test("autoImportProvider2", content);
    // TODO: f.MarkTestAsStradaServer()
    fourslash::go_to_marker(&mut s, "");
    // TODO: opts1155 := f.GetOptions()
    // TODO: opts1155.FormatCodeSettings.NewLineCharacter = "\n"
    // TODO: f.Configure(t, opts1155)
    // TODO: f.VerifyImportFixAtPosition(t, []string{}, nil /*preferences*/)
}
