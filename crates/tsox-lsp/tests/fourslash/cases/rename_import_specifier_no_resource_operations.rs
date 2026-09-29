use tsox_lsp::fourslash::{self, Session};


#[test]
fn rename_import_specifier_no_resource_operations() {
    let content = r#"
// @Filename: /a.ts
export const x = 0;
// @Filename: /b.ts
import * as a from ".//*rename*/a";"#;
    // TODO: capabilities := fourslash.GetDefaultCapabilities()
    // TODO: capabilities.Workspace.WorkspaceEdit = &lsproto.WorkspaceEditClientCapabilities{
    let mut s = Session::new_with_capabilities(content, None);
    fourslash::go_to_marker(&mut s, "rename");
    // TODO: f.VerifyRenameFailed(t, nil /*preferences*/)
}
