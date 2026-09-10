use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn rename_js_exports01() {
    let content = r#"// @allowJs: true
// @Filename: a.js
[|exports.[|{| "contextRangeIndex": 0 |}area|] = function (r) { return r * r; }|]
// @Filename: b.js
var mod = require('./a');
var t = mod./*1*/[|area|](10);"#;
    let mut s = Session::new_for_test("renameJsExports01", content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1")
    fourslash::unsupported("VerifyBaselineRenameAtRangesWithText"); // f.VerifyBaselineRenameAtRangesWithText(t, nil /*preferences*/, "area")
}
