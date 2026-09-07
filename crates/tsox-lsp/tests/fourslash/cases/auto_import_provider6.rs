use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: f.MarkTestAsStradaServer()"]
#[test]
fn auto_import_provider6() {
    let content = r#"// @Filename: /home/src/workspaces/project/tsconfig.json
{ "compilerOptions": { "module": "commonjs", "lib": ["es2019"], "types": ["*"] } }
// @Filename: /home/src/workspaces/project/package.json
{ "dependencies": { "antd": "*", "react": "*" } }
// @Filename: /home/src/workspaces/project/node_modules/@types/react/index.d.ts
export declare function Component(): void;
// @Filename: /home/src/workspaces/project/node_modules/antd/index.d.ts
import "react";
// @Filename: /home/src/workspaces/project/index.ts
Component/**/"#;
    let mut s = Session::new(content);
    // TODO: f.MarkTestAsStradaServer()
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
