use tsox_lsp::fourslash::{self, Session};


#[test]
fn auto_import_provider_namespace_same_name_as_intrinsic() {
    let content = r#"// @Filename: /home/src/workspaces/project/node_modules/fp-ts/package.json
{ "name": "fp-ts", "version": "0.10.4" }
// @Filename: /home/src/workspaces/project/node_modules/fp-ts/index.d.ts
export * as string from "./lib/string";
// @Filename: /home/src/workspaces/project/node_modules/fp-ts/lib/string.d.ts
export declare const fromString: (s: string) => string;
export type SafeString = string;
// @Filename: /home/src/workspaces/project/package.json
{ "dependencies": { "fp-ts": "^0.10.4" } }
// @Filename: /home/src/workspaces/project/tsconfig.json
{ "compilerOptions": { "module": "commonjs", "lib": ["es5"] } }
// @Filename: /home/src/workspaces/project/index.ts
type A = { name: string/**/ }"#;
    let mut s = Session::new_for_test("autoImportProvider_namespaceSameNameAsIntrinsic", content);
    // TODO: f.MarkTestAsStradaServer()
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
