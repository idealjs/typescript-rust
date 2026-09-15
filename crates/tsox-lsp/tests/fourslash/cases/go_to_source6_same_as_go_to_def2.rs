use tsox_lsp::fourslash::Session;


#[test]
fn go_to_source6_same_as_go_to_def2() {
    let content = r#"// @lib: es5
// @Filename: /home/src/workspaces/project/node_modules/foo/package.json
{ "name": "foo", "version": "1.2.3", "typesVersions": { "*": { "*": ["./types/*"] } } }
// @Filename: /home/src/workspaces/project/node_modules/foo/src/a.ts
export const /*end*/a = 'a';
// @Filename: /home/src/workspaces/project/node_modules/foo/types/a.d.ts
export declare const a: string;
//# sourceMappingURL=a.d.ts.map
// @Filename: /home/src/workspaces/project/node_modules/foo/types/a.d.ts.map
{"version":3,"file":"a.d.ts","sourceRoot":"","sources":["../src/a.ts"],"names":[],"mappings":"AAAA,eAAO,MAAM,EAAE,OAAO,CAAC;;AACvB,wBAAsB"}
// @Filename: /home/src/workspaces/project/node_modules/foo/dist/a.js
export const a = 'a';
// @Filename: /home/src/workspaces/project/b.ts
import { a } from 'foo/a';
[|a/*start*/|]"#;
    let _s = Session::new_for_test("goToSource6_sameAsGoToDef2", content);
    // TODO: f.MarkTestAsStradaServer()
    // TODO: f.VerifyBaselineGoToSourceDefinition(t, "start")
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "start")
}
