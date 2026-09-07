use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineRename"]
#[test]
fn rename_module_exports_properties3() {
    let content = r#"// @allowJs: true
// @Filename: a.js
[|class [|{| "contextRangeIndex": 0 |}A|] {}|]
module.exports = { [|A|] }"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineRename"); // f.VerifyBaselineRename(t, &lsutil.UserPreferences{UseAliasesForRename: core.TSTrue}, f.Ranges()[1], 
}
