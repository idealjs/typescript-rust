use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineRenameAtRangesWithText"]
#[test]
fn rename_js_property_assignment() {
    let content = r#"// @allowJs: true
// @Filename: a.js
function bar() {
}
[|bar.[|{| "contextRangeIndex": 0 |}foo|] = "foo";|]
console.log(bar.[|foo|]);"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineRenameAtRangesWithText"); // f.VerifyBaselineRenameAtRangesWithText(t, nil /*preferences*/, "foo")
}
