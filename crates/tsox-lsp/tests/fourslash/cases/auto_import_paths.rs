use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyImportFixModuleSpecifiers"]
#[test]
fn auto_import_paths() {
    let content = r#"// @Filename: /package1/jsconfig.json
{
  "compilerOptions": {
    checkJs: true,
    "paths": {
      "package1/*": ["./*"],
      "package2/*": ["../package2/*"]
    },
    "baseUrl": "."
  },
  "include": [
    ".",
    "../package2"
  ]
}
// @Filename: /package1/file1.js
bar/**/
// @Filename: /package2/file1.js
export const bar = 0;"#;
    let mut s = Session::new_for_test("autoImportPaths", content);
    fourslash::unsupported("VerifyImportFixModuleSpecifiers"); // f.VerifyImportFixModuleSpecifiers(t, "", []string{"package2/file1"}, &lsutil.UserPreferences{ImportM
}
