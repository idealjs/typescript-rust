use tsox_lsp::fourslash::{self, Session};


#[test]
fn auto_import_provider_wildcard_exports2() {
    let content = r#"// @Filename: /home/src/workspaces/project/node_modules/pkg/package.json
{
    "name": "pkg",
    "version": "1.0.0",
    "exports": {
        "./core/*": {
            "types": "./lib/core/*.d.ts",
            "default": "./lib/core/*.js"
        }
    }
}
// @Filename: /home/src/workspaces/project/node_modules/pkg/lib/core/test.d.ts
export function test(): void;
// @Filename: /home/src/workspaces/project/package.json
{
    "type": "module",
    "dependencies": {
        "pkg": "1.0.0"
    }
}
// @Filename: /home/src/workspaces/project/tsconfig.json
{
    "compilerOptions": {
        "module": "nodenext",
        "lib": ["es5"]
    }
}
// @Filename: /home/src/workspaces/project/main.ts
/**/"#;
    let mut s = Session::new_for_test("autoImportProvider_wildcardExports2", content);
    // TODO: f.MarkTestAsStradaServer()
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
