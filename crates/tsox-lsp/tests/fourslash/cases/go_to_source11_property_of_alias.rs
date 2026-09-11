use tsox_lsp::fourslash::{self, Session};


#[test]
fn go_to_source11_property_of_alias() {
    let content = r#"// @lib: es5
// @moduleResolution: bundler
// @Filename: /home/src/workspaces/project/a.js
export const a = { /*end*/a: 'a' };
// @Filename: /home/src/workspaces/project/a.d.ts
export declare const a: { a: string };
// @Filename: /home/src/workspaces/project/b.ts
import { a } from './a';
a.[|a/*start*/|]"#;
    let mut s = Session::new_for_test("goToSource11_propertyOfAlias", content);
    // TODO: f.MarkTestAsStradaServer()
    // TODO: f.VerifyBaselineGoToSourceDefinition(t, "start")
}
