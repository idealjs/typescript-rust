use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineRename"]
#[test]
fn rename_import_and_export() {
    let content = r#"[|import [|{| "contextRangeIndex": 0 |}a|] from "module";|]
[|export { [|{| "contextRangeIndex": 2 |}a|] };|]"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineRename"); // f.VerifyBaselineRename(t, nil /*preferences*/, f.Ranges()[1], f.Ranges()[3])
}
