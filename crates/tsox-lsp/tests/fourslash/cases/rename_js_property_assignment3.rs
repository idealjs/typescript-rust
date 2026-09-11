use tsox_lsp::fourslash::{self, Session};


#[test]
fn rename_js_property_assignment3() {
    let content = r#"// @allowJs: true
// @Filename: a.js
var C = class  {
}
[|C.[|{| "contextRangeIndex": 0 |}staticProperty|] = "string";|]
console.log(C.[|staticProperty|]);"#;
    let mut s = Session::new_for_test("renameJsPropertyAssignment3", content);
    // TODO: f.VerifyBaselineRenameAtRangesWithText(t, nil /*preferences*/, "staticProperty")
}
