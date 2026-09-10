use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineRenameAtRangesWithText"]
#[test]
fn rename_js_this_property06() {
    let content = r#"// @allowJs: true
// @Filename: a.js
var C = class {
  constructor(y) {
    this.x = y;
  }
}
[|C.prototype.[|{| "contextRangeIndex": 0 |}z|] = 1;|]
var t = new C(12);
[|t.[|{| "contextRangeIndex": 2 |}z|] = 11;|]"#;
    let mut s = Session::new_for_test("renameJsThisProperty06", content);
    fourslash::unsupported("VerifyBaselineRenameAtRangesWithText"); // f.VerifyBaselineRenameAtRangesWithText(t, nil /*preferences*/, "z")
}
