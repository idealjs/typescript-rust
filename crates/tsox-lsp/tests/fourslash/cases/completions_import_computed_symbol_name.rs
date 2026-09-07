use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: f.MarkTestAsStradaServer()"]
#[test]
fn completions_import_computed_symbol_name() {
    let content = r#"// @Filename: /home/src/workspaces/project/tsconfig.json
{ "compilerOptions": { "module": "commonjs", "lib": ["es5"] } }
// @Filename: /home/src/workspaces/project/node_modules/@types/ts-node/index.d.ts
export {};
declare const REGISTER_INSTANCE: unique symbol;
declare global {
    namespace NodeJS {
      interface Process {
          [REGISTER_INSTANCE]?: Service;
      }
  }
}
// @Filename: /home/src/workspaces/project/node_modules/@types/node/index.d.ts
declare module "process" {
    global {
        var process: NodeJS.Process;
        namespace NodeJS {
            interface Process {
                argv: string[];
            }
        }
    }
    export = process;
}
// @Filename: /home/src/workspaces/project/index.ts
I/**/"#;
    let mut s = Session::new(content);
    // TODO: f.MarkTestAsStradaServer()
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
    fourslash::insert(&mut s, "N");
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
