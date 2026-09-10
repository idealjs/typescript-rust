use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineRename"]
#[test]
fn rename_label5() {
    let content = r#"loop1: for (let i = 0; i <= 10; i++) {
    loop2: for (let j = 0; j <= 10; j++) {
        if (i === 5) continue /**/loop1;
        if (j === 5) break loop2;
    }
}"#;
    let mut s = Session::new_for_test("renameLabel5", content);
    fourslash::unsupported("VerifyBaselineRename"); // f.VerifyBaselineRename(t, nil /*preferences*/, "")
}
