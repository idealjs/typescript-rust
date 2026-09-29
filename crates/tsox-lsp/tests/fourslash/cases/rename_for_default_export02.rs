use tsox_lsp::fourslash::Session;


#[test]
fn rename_for_default_export02() {
    let content = r#"[|export default function /*1*/[|{| "contextRangeIndex": 0 |}DefaultExportedFunction|]() {
    return /*2*/[|DefaultExportedFunction|]
}|]
/**
 *  Commenting [|{| "inComment": true |}DefaultExportedFunction|]
 */

var x: typeof /*3*/[|DefaultExportedFunction|];

var y = /*4*/[|DefaultExportedFunction|]();"#;
    let _s = Session::new_for_test("renameForDefaultExport02", content);
    // TODO: f.VerifyBaselineRename(t, nil /*preferences*/, ToAny(core.Filter(f.GetRangesByText().Get("DefaultExp
}
