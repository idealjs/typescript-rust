use tsox_lsp::fourslash::Session;


#[test]
fn rename_js_property_assignment2() {
    let content = r#"// @allowJs: true
// @Filename: a.js
class Minimatch {
}
[|Minimatch.[|{| "contextRangeIndex": 0 |}staticProperty|] = "string";|]
console.log(Minimatch.[|staticProperty|]);"#;
    let _s = Session::new_for_test("renameJsPropertyAssignment2", content);
    // TODO: f.VerifyBaselineRenameAtRangesWithText(t, nil /*preferences*/, "staticProperty")
}
