use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: f.MarkTestAsStradaServer()"]
#[test]
fn completions_import_merged_re_export() {
    let content = r#"// @Filename: /home/src/workspaces/project/tsconfig.json
{ "compilerOptions": { "module": "commonjs", "lib": ["es5"] } }
// @Filename: /home/src/workspaces/project/package.json
{ "dependencies": { "@jest/types": "*", "ts-jest": "*" } }
// @Filename: /home/src/workspaces/project/node_modules/@jest/types/package.json
{ "name": "@jest/types" }
// @Filename: /home/src/workspaces/project/node_modules/@jest/types/index.d.ts
import type * as Config from "./Config";
export type { Config };
// @Filename: /home/src/workspaces/project/node_modules/@jest/types/Config.d.ts
export interface ConfigGlobals {
    [K: string]: unknown;
}
// @Filename: /home/src/workspaces/project/node_modules/ts-jest/index.d.ts
export {};
declare module "@jest/types" {
    namespace Config {
        interface ConfigGlobals {
            'ts-jest': any;
        }
    }
}
// @Filename: /home/src/workspaces/project/index.ts
C/**/"#;
    let mut s = Session::new_for_test("completionsImport_mergedReExport", content);
    // TODO: f.MarkTestAsStradaServer()
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
    fourslash::insert(&mut s, "o");
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
