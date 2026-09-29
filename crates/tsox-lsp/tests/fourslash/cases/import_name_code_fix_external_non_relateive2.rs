use tsox_lsp::fourslash::{self, Session};


#[test]
fn import_name_code_fix_external_non_relateive2() {
    let content = r#"// @Filename: /home/src/workspaces/project/apps/app1/tsconfig.json
{
  "compilerOptions": {
    "module": "commonjs",
    "lib": ["es5"],
    "paths": {
      "shared/*": ["../../shared/*"]
    }
  },
  "include": ["src", "../../shared"]
}
// @Filename: /home/src/workspaces/project/apps/app1/src/index.ts
shared/*internal2external*/
// @Filename: /home/src/workspaces/project/apps/app1/src/app.ts
utils/*internal2internal*/
// @Filename: /home/src/workspaces/project/apps/app1/src/utils.ts
export const utils = 0;
// @Filename: /home/src/workspaces/project/shared/constants.ts
export const shared = 0;
// @Filename: /home/src/workspaces/project/shared/data.ts
shared/*external2external*/"#;
    let mut s = Session::new_for_test("importNameCodeFix_externalNonRelateive2", content);
    // TODO: f.MarkTestAsStradaServer()
    // TODO: opts839 := f.GetOptions()
    // TODO: opts839.FormatCodeSettings.NewLineCharacter = "\n"
    // TODO: f.Configure(t, opts839)
    fourslash::go_to_marker(&mut s, "internal2external");
    // TODO: f.VerifyImportFixAtPosition(t, []string{
    fourslash::go_to_marker(&mut s, "internal2internal");
    // TODO: f.VerifyImportFixAtPosition(t, []string{
    fourslash::go_to_marker(&mut s, "external2external");
    // TODO: f.VerifyImportFixAtPosition(t, []string{
}
