use tsox_lsp::fourslash::{self, Session};


#[test]
fn rename_js_exports01() {
    let content = r#"// @allowJs: true
// @Filename: a.js
[|exports.[|{| "contextRangeIndex": 0 |}area|] = function (r) { return r * r; }|]
// @Filename: b.js
var mod = require('./a');
var t = mod./*1*/[|area|](10);"#;
    let mut s = Session::new_for_test("renameJsExports01", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1")
    // TODO: f.VerifyBaselineRenameAtRangesWithText(t, nil /*preferences*/, "area")
}
