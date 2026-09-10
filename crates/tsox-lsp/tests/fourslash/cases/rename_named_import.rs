use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: f.MarkTestAsStradaServer()"]
#[test]
fn rename_named_import() {
    let content = r#"// @Filename: /home/src/workspaces/project/lib/tsconfig.json
{ "compilerOptions": { "lib": ["es5"] } }
// @Filename: /home/src/workspaces/project/lib/index.ts
const unrelatedLocalVariable = 123;
export const someExportedVariable = unrelatedLocalVariable;
// @Filename: /home/src/workspaces/project/src/tsconfig.json
{ "compilerOptions": { "lib": ["es5"] } }
// @Filename: /home/src/workspaces/project/src/index.ts
import { /*i*/someExportedVariable } from '../lib/index';
someExportedVariable;
// @Filename: /home/src/workspaces/project/tsconfig.json
{ "compilerOptions": { "lib": ["es5"] } }"#;
    let mut s = Session::new_for_test("renameNamedImport", content);
    // TODO: f.MarkTestAsStradaServer()
    fourslash::go_to_file(&mut s, "/home/src/workspaces/project/lib/index.ts");
    fourslash::go_to_file(&mut s, "/home/src/workspaces/project/src/index.ts");
    fourslash::unsupported("VerifyBaselineRename"); // f.VerifyBaselineRename(t, &lsutil.UserPreferences{UseAliasesForRename: core.TSTrue}, "i")
}
