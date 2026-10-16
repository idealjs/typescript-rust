use tsox_lsp::fourslash::Session;


#[test]
fn get_edits_for_file_rename_css_import2() {
    let content = r#"// @Filename: /tsconfig.json
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
import styles from "./app.css";"#;
    let _s = Session::new_for_test("getEditsForFileRename_cssImport2", content);
    // TODO: f.VerifyWillRenameFilesEdits(t, "/app.d.css.ts", "/app2.d.css.ts", map[string]string{
}
