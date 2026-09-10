use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: f.MarkTestAsStradaServer()"]
#[test]
fn import_name_code_fix_external_non_relative1() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @Filename: /home/src/workspaces/project/tsconfig.base.json
{
  "compilerOptions": {
    "module": "commonjs",
    "lib": ["es5"],
    "paths": {
      "pkg-1/*": ["./packages/pkg-1/src/*"],
      "pkg-2/*": ["./packages/pkg-2/src/*"]
    }
  }
}
// @Filename: /home/src/workspaces/project/packages/pkg-1/package.json
{ "dependencies": { "pkg-2": "*" } }
// @Filename: /home/src/workspaces/project/packages/pkg-1/tsconfig.json
{
  "extends": "../../tsconfig.base.json",
  "references": [
    { "path": "../pkg-2" }
  ]
}
// @Filename: /home/src/workspaces/project/packages/pkg-1/src/index.ts
Pkg2/*external*/
// @Filename: /home/src/workspaces/project/packages/pkg-2/package.json
{ "types": "dist/index.d.ts" }
// @Filename: /home/src/workspaces/project/packages/pkg-2/tsconfig.json
{
  "extends": "../../tsconfig.base.json",
  "compilerOptions": { "outDir": "dist", "rootDir": "src", "composite": true, "lib": ["es5"] }
}
// @Filename: /home/src/workspaces/project/packages/pkg-2/src/index.ts
import "./utils";
// @Filename: /home/src/workspaces/project/packages/pkg-2/src/utils.ts
export const Pkg2 = {};
// @Filename: /home/src/workspaces/project/packages/pkg-2/src/blah/foo/data.ts
Pkg2/*internal*/
// @link: /home/src/workspaces/project/packages/pkg-2 -> /home/src/workspaces/project/packages/pkg-1/node_modules/pkg-2"#;
    let mut s = Session::new_for_test("importNameCodeFix_externalNonRelative1", content);
    // TODO: f.MarkTestAsStradaServer()
    // TODO: opts1534 := f.GetOptions()
    // TODO: opts1534.FormatCodeSettings.NewLineCharacter = "\n"
    fourslash::unsupported("Configure"); // f.Configure(t, opts1534)
    fourslash::go_to_marker(&mut s, "external");
    fourslash::unsupported("VerifyImportFixAtPosition"); // f.VerifyImportFixAtPosition(t, []string{
    fourslash::go_to_marker(&mut s, "internal");
    fourslash::unsupported("VerifyImportFixAtPosition"); // f.VerifyImportFixAtPosition(t, []string{
}
