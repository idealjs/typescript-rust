use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyWillRenameFilesEdits"]
#[test]
fn get_edits_for_file_rename_unresolvable_import() {
    let content = r#"// @Filename: /tsconfig.json
{
  "compilerOptions": {
    "allowJs": true,
    "paths": {
      "*": ["./next/src/*"],
      "@app": ["./modules/@app/*"],
      "@app/*": ["./modules/@app/*"],
      "@local": ["./modules/@local/*"],
      "@local/*": ["./modules/@local/*"]
    }
  }
}
// @Filename: /modules/@app/something/index.js
import "@local/some-other-import";
// @Filename: /modules/@local/index.js
import "@local/some-other-import";"#;
    let mut s = Session::new_for_test("getEditsForFileRename_unresolvableImport", content);
    fourslash::unsupported("VerifyWillRenameFilesEdits"); // f.VerifyWillRenameFilesEdits(t, "/modules/@app/something", "/modules/@app/something-2", map[string]s
}
