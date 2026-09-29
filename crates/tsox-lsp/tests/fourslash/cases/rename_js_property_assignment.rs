use tsox_lsp::fourslash::Session;


#[test]
fn rename_js_property_assignment() {
    let content = r#"// @allowJs: true
// @Filename: a.js
function bar() {
}
[|bar.[|{| "contextRangeIndex": 0 |}foo|] = "foo";|]
console.log(bar.[|foo|]);"#;
    let _s = Session::new_for_test("renameJsPropertyAssignment", content);
    // TODO: f.VerifyBaselineRenameAtRangesWithText(t, nil /*preferences*/, "foo")
}
