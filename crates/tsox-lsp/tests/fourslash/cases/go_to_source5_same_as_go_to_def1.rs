use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: f.MarkTestAsStradaServer()"]
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
    let mut s = Session::new(content);
    // TODO: f.MarkTestAsStradaServer()
    fourslash::unsupported("VerifyBaselineGoToSourceDefinition"); // f.VerifyBaselineGoToSourceDefinition(t, "start")
    fourslash::unsupported("VerifyBaselineGoToDefinition"); // f.VerifyBaselineGoToDefinition(t, true, "start")
}
