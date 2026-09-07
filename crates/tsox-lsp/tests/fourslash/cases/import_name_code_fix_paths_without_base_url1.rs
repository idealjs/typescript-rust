use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyImportFixAtPosition"]
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
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("VerifyImportFixAtPosition"); // f.VerifyImportFixAtPosition(t, []string{
}
