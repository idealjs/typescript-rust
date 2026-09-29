use tsox_lsp::fourslash::{self, Session};


#[test]
fn import_name_code_fix_paths_without_base_url1() {
    let content = r#"// @Filename: tsconfig.json
{
  "compilerOptions": {
    "module": "commonjs",
    "paths": {
      "@app/*": ["./lib/*"]
    }
  }
}
// @Filename: index.ts
utils/**/
// @Filename: lib/utils.ts
export const utils = {};"#;
    let mut s = Session::new_for_test("importNameCodeFix_pathsWithoutBaseUrl1", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyImportFixAtPosition(t, []string{
}
