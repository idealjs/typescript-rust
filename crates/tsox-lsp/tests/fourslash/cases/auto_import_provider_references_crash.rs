use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: f.MarkTestAsStradaServer()"]
#[test]
fn auto_import_provider_references_crash() {
    let content = r#"// @Filename: /home/src/workspaces/project/a/package.json
{}
// @Filename: /home/src/workspaces/project/a/tsconfig.json
{ "compilerOptions": { "lib": ["es5"] } }
// @Filename: /home/src/workspaces/project/a/index.ts
class A {}
// @Filename: /home/src/workspaces/project/a/index.d.ts
declare class A {
}
//# sourceMappingURL=index.d.ts.map
// @Filename: /home/src/workspaces/project/a/index.d.ts.map
{"version":3,"file":"index.d.ts","sourceRoot":"","sources":["index.ts"],"names":[],"mappings":"AAAA,OAAO,OAAO,CAAC;CAAG"}
// @Filename: /home/src/workspaces/project/b/tsconfig.json
{
  "compilerOptions": { "disableSourceOfProjectReferenceRedirect": true, "lib": ["es5"] },
  "references": [{ "path": "../a" }]
}
// @Filename: /home/src/workspaces/project/b/b.ts
/// <reference path="../a/index.d.ts" />
new A/**/();
// @Filename: /home/src/workspaces/project/c/package.json
{ "dependencies": { "a": "*" } }
// @Filename: /home/src/workspaces/project/c/tsconfig.json
{ "compilerOptions": { "lib": ["es5"] }, "references" [{ "path": "../a" }] }
// @Filename: /home/src/workspaces/project/c/index.ts
export {};
// @link: /home/src/workspaces/project/a -> /home/src/workspaces/project/c/node_modules/a"#;
    let mut s = Session::new(content);
    // TODO: f.MarkTestAsStradaServer()
    fourslash::go_to_file(&mut s, "/home/src/workspaces/project/c/index.ts");
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "")
}
