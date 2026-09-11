use tsox_lsp::fourslash::{self, Session};


#[test]
fn rename_comments_and_strings1() {
    let content = r#"///<reference path="./Bar.ts" />
[|function [|{| "contextRangeIndex": 0 |}Bar|]() {
    // This is a reference to Bar in a comment.
    "this is a reference to Bar in a string"
}|]"#;
    let mut s = Session::new_for_test("renameCommentsAndStrings1", content);
    // TODO: f.VerifyBaselineRenameAtRangesWithText(t, nil /*preferences*/, "Bar")
}
