use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: f.MarkTestAsStradaServer()"]
#[test]
fn go_to_source1_local_js_beside_dts() {
    let content = r#"// @lib: es5
// @Filename: /home/src/workspaces/project/a.js
export const /*end*/a = "a";
// @Filename: /home/src/workspaces/project/a.d.ts
export declare const a: string;
// @Filename: /home/src/workspaces/project/index.ts
import { a } from [|"./a"/*moduleSpecifier*/|];
[|a/*identifier*/|]"#;
    let mut s = Session::new(content);
    // TODO: f.MarkTestAsStradaServer()
    fourslash::unsupported("VerifyBaselineGoToSourceDefinition"); // f.VerifyBaselineGoToSourceDefinition(t, "identifier", "moduleSpecifier")
}
