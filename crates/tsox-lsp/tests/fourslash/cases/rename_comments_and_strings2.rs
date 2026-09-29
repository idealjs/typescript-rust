use tsox_lsp::fourslash::Session;


#[test]
fn rename_comments_and_strings2() {
    let content = r#"///<reference path="./Bar.ts" />
[|function [|{| "contextRangeIndex": 0 |}Bar|]() {
    // This is a reference to Bar in a comment.
    "this is a reference to [|Bar|] in a string"
}|]"#;
    let _s = Session::new_for_test("renameCommentsAndStrings2", content);
    // TODO: f.VerifyBaselineRename(t, nil /*preferences*/, f.Ranges()[1])
}
