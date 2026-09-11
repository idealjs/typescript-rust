use tsox_lsp::fourslash::{self, Session};


#[test]
fn rename_module_exports_properties2() {
    let content = r#"[|class [|{| "contextRangeIndex": 0 |}A|] {}|]
module.exports = { B: [|A|] }"#;
    let mut s = Session::new_for_test("renameModuleExportsProperties2", content);
    // TODO: f.VerifyBaselineRename(t, nil /*preferences*/, f.Ranges()[1], f.Ranges()[2])
}
