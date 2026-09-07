use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyImportFixAtPosition"]
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
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("VerifyImportFixAtPosition"); // f.VerifyImportFixAtPosition(t, []string{
}
