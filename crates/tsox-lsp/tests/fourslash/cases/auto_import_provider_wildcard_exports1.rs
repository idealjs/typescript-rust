use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: f.MarkTestAsStradaServer()"]
#[test]
fn auto_import_provider_wildcard_exports1() {
    let content = r#"// @Filename: /home/src/workspaces/project/node_modules/pkg/package.json
{
    "name": "pkg",
    "version": "1.0.0",
    "exports": {
        "./*": "./a/*.js",
        "./b/*.js": "./b/*.js",
        "./c/*": "./c/*",
        "./d/*": {
            "import": "./d/*.mjs"
        }
    }
}
// @Filename: /home/src/workspaces/project/node_modules/pkg/a/a1.d.ts
export const a1: number;
// @Filename: /home/src/workspaces/project/node_modules/pkg/b/b1.d.ts
export const b1: number;
// @Filename: /home/src/workspaces/project/node_modules/pkg/b/b2.d.mts
export const NOT_REACHABLE: number;
// @Filename: /home/src/workspaces/project/node_modules/pkg/c/c1.d.ts
export const c1: number;
// @Filename: /home/src/workspaces/project/node_modules/pkg/c/subfolder/c2.d.mts
export const c2: number;
// @Filename: /home/src/workspaces/project/node_modules/pkg/d/d1.d.mts
export const d1: number;
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
    let mut s = Session::new(content);
    // TODO: f.MarkTestAsStradaServer()
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
