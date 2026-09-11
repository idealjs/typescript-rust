use tsox_lsp::fourslash::{self, Session};


#[test]
fn import_statement_completions_pnpm_transitive() {
    let content = r#"// @Filename: /home/src/workspaces/project/tsconfig.json
{ "compilerOptions": { "module": "commonjs", "lib": ["es5"] } }
// @Filename: /home/src/workspaces/project/node_modules/.pnpm/@types+react@17.0.7/node_modules/@types/react/index.d.ts
import "csstype";
export declare function Component(): void;
// @Filename: /home/src/workspaces/project/node_modules/.pnpm/csstype@3.0.8/node_modules/csstype/index.d.ts
export interface SvgProperties {}
// @Filename: /home/src/workspaces/project/index.ts
[|import SvgProp/**/|]
// @link: /home/src/workspaces/project/node_modules/.pnpm/@types+react@17.0.7/node_modules/@types/react -> /home/src/workspaces/project/node_modules/@types/react
// @link: /home/src/workspaces/project/node_modules/.pnpm/csstype@3.0.8/node_modules/csstype -> /home/src/workspaces/project/node_modules/.pnpm/@types+react@17.0.7/node_modules/csstype"#;
    let mut s = Session::new_with_capabilities(content, None);
    // TODO: f.MarkTestAsStradaServer()
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
