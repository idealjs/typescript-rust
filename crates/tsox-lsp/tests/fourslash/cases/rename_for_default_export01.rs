use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: for _, r := range ranges {"]
#[test]
fn rename_for_default_export01() {
    let content = r#"[|export default class [|{| "contextRangeIndex": 0 |}DefaultExportedClass|] {
}|]
/*
 *  Commenting [|{| "inComment": true |}DefaultExportedClass|]
 */

var x: [|DefaultExportedClass|];

var y = new [|DefaultExportedClass|];"#;
    let mut s = Session::new(content);
    // TODO: ranges := f.GetRangesByText().Get("DefaultExportedClass")
    // TODO: var markerOrRanges []fourslash.MarkerOrRangeOrName
    // TODO: for _, r := range ranges {
    fourslash::unsupported("VerifyBaselineRename"); // f.VerifyBaselineRename(t, nil /*preferences*/, markerOrRanges...)
}
