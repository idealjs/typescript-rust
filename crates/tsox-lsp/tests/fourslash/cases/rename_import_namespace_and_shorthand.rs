use tsox_lsp::fourslash::Session;


#[test]
fn rename_import_namespace_and_shorthand() {
    let content = r#"[|import * as [|{| "contextRangeIndex": 0 |}foo|] from 'bar';|]
const bar = { [|foo|] };"#;
    let _s = Session::new_for_test("renameImportNamespaceAndShorthand", content);
    // TODO: f.VerifyBaselineRename(t, nil /*preferences*/, f.Ranges()[1], f.Ranges()[2])
}
