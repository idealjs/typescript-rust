use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyRenameSucceeded"]
#[test]
fn rename_name_on_enum_member() {
    let content = r#"enum e {
    firstMember,
    secondMember,
    thirdMember
}
var enumMember = e.[|/**/thirdMember|];"#;
    let mut s = Session::new_for_test("renameNameOnEnumMember", content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("VerifyRenameSucceeded"); // f.VerifyRenameSucceeded(t, nil /*preferences*/)
}
