use tsox_lsp::fourslash::{self, Session};


#[test]
fn get_edits_for_file_rename_css_import4() {
    let content = r#"
// @Filename: /tsconfig.json
{ "compilerOptions": { "allowArbitraryExtensions": true } }
// @Filename: /app.css
.cookie-banner {
  display: none;
}
// @Filename: /app.d.css.ts
declare const css: {
  cookieBanner: string;
};
export default css;
// @Filename: /a.ts
import styles from ".//*rename*/app.css";"#;
    // TODO: capabilities := fourslash.GetDefaultCapabilities()
    // TODO: capabilities.Workspace.FileOperations.WillRename = new(false)
    let mut s = Session::new_with_capabilities(content, None);
    // TODO: f.VerifyRename(t, "rename", "app2.css", map[string]string{
}
