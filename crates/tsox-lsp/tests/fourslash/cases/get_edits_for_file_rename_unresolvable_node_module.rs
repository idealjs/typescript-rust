use tsox_lsp::fourslash::Session;


#[test]
fn get_edits_for_file_rename_unresolvable_node_module() {
    let content = r#"// @allowJs: true
// @checkJs: true
// @Filename: /modules/@app/something/index.js
import "doesnt-exist";
// @Filename: /modules/@local/foo.js
import "doesnt-exist"; "#;
    let _s = Session::new_for_test("getEditsForFileRename_unresolvableNodeModule", content);
    // TODO: f.VerifyWillRenameFilesEdits(t, "/modules/@app/something", "/modules/@app/something-2", map[string]s
}
