use tsox_lsp::fourslash::Session;


#[test]
fn rename_label3() {
    let content = r#"/**/loop:
for (let i = 0; i <= 10; i++) {
   if (i === 0) continue loop;
   if (i === 1) continue loop;
   if (i === 10) break loop;
}"#;
    let _s = Session::new_for_test("renameLabel3", content);
    // TODO: f.VerifyBaselineRename(t, nil /*preferences*/, "")
}
