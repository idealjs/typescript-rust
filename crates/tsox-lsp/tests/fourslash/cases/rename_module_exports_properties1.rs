use tsox_lsp::fourslash::{self, Session};


#[test]
fn rename_module_exports_properties1() {
    let content = r#"[|class [|{| "contextRangeIndex": 0 |}A|] {}|]
module.exports = { [|A|] }"#;
    let mut s = Session::new_for_test("renameModuleExportsProperties1", content);
    // TODO: f.VerifyBaselineRename(t, &lsutil.UserPreferences{UseAliasesForRename: core.TSTrue}, f.Ranges()[1], 
}
