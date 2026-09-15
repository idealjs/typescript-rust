use tsox_lsp::fourslash::Session;


#[test]
fn rename_js_this_property05() {
    let content = r#"// @allowJs: true
// @Filename: a.js
class C {
  constructor(y) {
    this.x = y;
  }
}
[|C.prototype.[|{| "contextRangeIndex": 0 |}z|] = 1;|]
var t = new C(12);
[|t.[|{| "contextRangeIndex": 2 |}z|] = 11;|]"#;
    let _s = Session::new_for_test("renameJsThisProperty05", content);
    // TODO: f.VerifyBaselineRenameAtRangesWithText(t, nil /*preferences*/, "z")
}
