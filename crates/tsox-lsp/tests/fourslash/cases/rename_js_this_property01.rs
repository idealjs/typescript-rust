use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineRenameAtRangesWithText"]
#[test]
fn rename_js_this_property01() {
    let content = r#"// @allowJs: true
// @Filename: a.js
function bar() {
    [|this.[|{| "contextRangeIndex": 0 |}x|] = 10;|]
}
var t = new bar();
[|t.[|{| "contextRangeIndex": 2 |}x|] = 11;|]"#;
    let mut s = Session::new_for_test("renameJsThisProperty01", content);
    fourslash::unsupported("VerifyBaselineRenameAtRangesWithText"); // f.VerifyBaselineRenameAtRangesWithText(t, nil /*preferences*/, "x")
}
