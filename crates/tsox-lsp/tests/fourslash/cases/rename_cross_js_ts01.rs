use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineRename"]
#[test]
fn rename_cross_js_ts01() {
    let content = r#"// @allowJs: true
// @Filename: a.js
[|exports.[|{| "contextRangeIndex": 0 |}area|] = function (r) { return r * r; }|]
// @Filename: b.ts
[|import { [|{| "contextRangeIndex": 2 |}area|] } from './a';|]
var t = [|area|](10);"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineRename"); // f.VerifyBaselineRename(t, nil /*preferences*/, f.Ranges()[1], f.Ranges()[3], f.Ranges()[4])
}
