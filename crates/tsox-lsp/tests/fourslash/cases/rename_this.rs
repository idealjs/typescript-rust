use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineRename"]
#[test]
fn rename_this() {
    let content = r#"function f([|this|]) {
    return [|this|];
}
this/**/;
const _ = { [|[|{| "contextRangeIndex": 2 |}this|]: 0|] }.[|this|];"#;
    let mut s = Session::new_for_test("renameThis", content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("VerifyRenameFailed"); // f.VerifyRenameFailed(t, nil /*preferences*/)
    fourslash::unsupported("VerifyBaselineRename"); // f.VerifyBaselineRename(t, nil /*preferences*/, f.Ranges()[0], f.Ranges()[1], f.Ranges()[3], f.Ranges
}
