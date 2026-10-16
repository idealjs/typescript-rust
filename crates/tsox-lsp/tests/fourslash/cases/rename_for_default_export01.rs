use tsox_lsp::fourslash::Session;


#[test]
fn rename_for_default_export01() {
    let content = r#"[|export default class [|{| "contextRangeIndex": 0 |}DefaultExportedClass|] {
}|]
/*
 *  Commenting [|{| "inComment": true |}DefaultExportedClass|]
 */

var x: [|DefaultExportedClass|];

var y = new [|DefaultExportedClass|];"#;
    let _s = Session::new_for_test("renameForDefaultExport01", content);
    // TODO: ranges := f.GetRangesByText().Get("DefaultExportedClass")
    // TODO: var markerOrRanges []fourslash.MarkerOrRangeOrName
    // TODO: for _, r := range ranges {
    // TODO: f.VerifyBaselineRename(t, nil /*preferences*/, markerOrRanges...)
}
