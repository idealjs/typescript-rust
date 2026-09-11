use tsox_lsp::fourslash::{self, Session};


#[test]
fn go_to_source5_same_as_go_to_def1() {
    let content = r#"// @lib: es5
// @Filename: /home/src/workspaces/project/a.ts
export const /*end*/a = 'a';
// @Filename: /home/src/workspaces/project/a.d.ts
export declare const a: string;
// @Filename: /home/src/workspaces/project/a.js
export const a = 'a';
// @Filename: /home/src/workspaces/project/b.ts
import { a } from './a';
[|a/*start*/|]"#;
    let mut s = Session::new_for_test("goToSource5_sameAsGoToDef1", content);
    // TODO: f.MarkTestAsStradaServer()
    // TODO: f.VerifyBaselineGoToSourceDefinition(t, "start")
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "start")
}
