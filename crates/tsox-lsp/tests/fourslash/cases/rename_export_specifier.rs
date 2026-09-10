use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineRename"]
#[test]
fn rename_export_specifier() {
    let content = r#"// @Filename: a.ts
const name = {};
export { name as name/**/ };
// @Filename: b.ts
import { name } from './a';
const x = name.toString();"#;
    let mut s = Session::new_for_test("renameExportSpecifier", content);
    fourslash::unsupported("VerifyBaselineRename"); // f.VerifyBaselineRename(t, &lsutil.UserPreferences{UseAliasesForRename: core.TSFalse}, "")
}
