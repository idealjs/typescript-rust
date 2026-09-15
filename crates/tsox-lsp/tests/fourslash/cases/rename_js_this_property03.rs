use tsox_lsp::fourslash::Session;


#[test]
fn rename_js_this_property03() {
    let content = r#"// @allowJs: true
// @Filename: a.js
class C {
  constructor(y) {
    [|this.[|{| "contextRangeIndex": 0 |}x|] = y;|]
  }
}
var t = new C(12);
[|t.[|{| "contextRangeIndex": 2 |}x|] = 11;|]"#;
    let _s = Session::new_for_test("renameJsThisProperty03", content);
    // TODO: f.VerifyBaselineRenameAtRangesWithText(t, nil /*preferences*/, "x")
}
