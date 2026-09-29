use tsox_lsp::fourslash::Session;


#[test]
fn rename_import_specifier_property_name() {
    let content = r#"// @Filename: canada.ts
export interface /**/Ginger {}
// @Filename: dry.ts
import { Ginger as Ale } from './canada';"#;
    let _s = Session::new_for_test("renameImportSpecifierPropertyName", content);
    // TODO: f.VerifyBaselineRename(t, nil /*preferences*/, "")
}
