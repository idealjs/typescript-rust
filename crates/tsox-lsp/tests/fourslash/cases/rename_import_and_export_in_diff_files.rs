use tsox_lsp::fourslash::Session;


#[test]
fn rename_import_and_export_in_diff_files() {
    let content = r#"// @Filename: a.ts
[|export var /*1*/[|{| "isDefinition": true, "contextRangeIndex": 0 |}a|];|]
// @Filename: b.ts
[|import { /*2*/[|{| "isWriteAccess": true, "isDefinition": true, "contextRangeIndex": 2 |}a|] } from './a';|]
[|export { /*3*/[|{| "isWriteAccess": true, "isDefinition": true, "contextRangeIndex": 4 |}a|] };|]"#;
    let _s = Session::new_for_test("renameImportAndExportInDiffFiles", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2", "3")
    // TODO: f.VerifyBaselineRename(t, nil /*preferences*/, f.Ranges()[1], f.Ranges()[3], f.Ranges()[5])
}
