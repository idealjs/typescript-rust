use tsox_lsp::fourslash::Session;


#[test]
fn rename_label5() {
    let content = r#"loop1: for (let i = 0; i <= 10; i++) {
    loop2: for (let j = 0; j <= 10; j++) {
        if (i === 5) continue /**/loop1;
        if (j === 5) break loop2;
    }
}"#;
    let _s = Session::new_for_test("renameLabel5", content);
    // TODO: f.VerifyBaselineRename(t, nil /*preferences*/, "")
}
