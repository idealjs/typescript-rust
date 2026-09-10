use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineRename"]
#[test]
fn rename_import_and_shorthand() {
    let content = r#"[|import [|{| "contextRangeIndex": 0 |}foo|] from 'bar';|]
const bar = { [|foo|] };"#;
    let mut s = Session::new_for_test("renameImportAndShorthand", content);
    fourslash::unsupported("VerifyBaselineRename"); // f.VerifyBaselineRename(t, nil /*preferences*/, f.Ranges()[1], f.Ranges()[2])
}
