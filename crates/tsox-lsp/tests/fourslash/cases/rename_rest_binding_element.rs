use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineRename"]
#[test]
fn rename_rest_binding_element() {
    let content = r#"interface I {
    a: number;
    b: number;
    c: number;
}
function foo([|{ a, ...[|{| "contextRangeIndex": 0 |}rest|] }: I|]) {
    [|rest|];
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineRename"); // f.VerifyBaselineRename(t, &lsutil.UserPreferences{UseAliasesForRename: core.TSTrue}, f.Ranges()[1])
}
