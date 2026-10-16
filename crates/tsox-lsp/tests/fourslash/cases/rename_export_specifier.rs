use tsox_lsp::fourslash::Session;


#[test]
fn rename_export_specifier() {
    let content = r#"// @Filename: a.ts
const name = {};
export { name as name/**/ };
// @Filename: b.ts
import { name } from './a';
const x = name.toString();"#;
    let _s = Session::new_for_test("renameExportSpecifier", content);
    // TODO: f.VerifyBaselineRename(t, &lsutil.UserPreferences{UseAliasesForRename: core.TSFalse}, "")
}
