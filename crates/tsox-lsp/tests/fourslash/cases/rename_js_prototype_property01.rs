use tsox_lsp::fourslash::{self, Session};


#[test]
fn rename_js_prototype_property01() {
    let content = r#"// @allowJs: true
// @Filename: a.js
function bar() {
}
[|bar.prototype.[|{| "contextRangeIndex": 0 |}x|] = 10;|]
var t = new bar();
[|t.[|{| "contextRangeIndex": 2 |}x|] = 11;|]"#;
    let mut s = Session::new_for_test("renameJsPrototypeProperty01", content);
    // TODO: f.VerifyBaselineRenameAtRangesWithText(t, nil /*preferences*/, "x")
}
