use tsox_lsp::fourslash::Session;


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
    let _s = Session::new_for_test("renameRestBindingElement", content);
    // TODO: f.VerifyBaselineRename(t, &lsutil.UserPreferences{UseAliasesForRename: core.TSTrue}, f.Ranges()[1])
}
