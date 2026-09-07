use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyWillRenameFilesEdits"]
#[test]
fn get_edits_for_file_rename_unresolvable_node_module() {
    let content = r#"// @allowJs: true
// @checkJs: true
// @Filename: /modules/@app/something/index.js
import "doesnt-exist";
// @Filename: /modules/@local/foo.js
import "doesnt-exist"; "#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyWillRenameFilesEdits"); // f.VerifyWillRenameFilesEdits(t, "/modules/@app/something", "/modules/@app/something-2", map[string]s
}
