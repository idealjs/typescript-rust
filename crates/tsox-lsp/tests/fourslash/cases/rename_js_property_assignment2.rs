use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineRenameAtRangesWithText"]
#[test]
fn rename_js_property_assignment2() {
    let content = r#"// @allowJs: true
// @Filename: a.js
class Minimatch {
}
[|Minimatch.[|{| "contextRangeIndex": 0 |}staticProperty|] = "string";|]
console.log(Minimatch.[|staticProperty|]);"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineRenameAtRangesWithText"); // f.VerifyBaselineRenameAtRangesWithText(t, nil /*preferences*/, "staticProperty")
}
