use tsox_lsp::fourslash::{self, Session};


#[test]
fn import_name_code_fix_paths_without_base_url2() {
    let content = r#"// @Filename: /packages/test-package-1/tsconfig.json
{
  "compilerOptions": {
    "module": "commonjs",
    "paths": {
      "test-package-2/*": ["../test-package-2/src/*"]
    }
  }
}
// @Filename: /packages/test-package-1/src/common/logging.ts
export class Logger {};
// @Filename: /packages/test-package-1/src/something/index.ts
Logger/**/"#;
    let mut s = Session::new_for_test("importNameCodeFix_pathsWithoutBaseUrl2", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyImportFixAtPosition(t, []string{
}
