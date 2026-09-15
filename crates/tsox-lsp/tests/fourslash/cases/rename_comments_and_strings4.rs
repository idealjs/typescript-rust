use tsox_lsp::fourslash::Session;


#[test]
fn rename_comments_and_strings4() {
    let content = r#"///<reference path="./Bar.ts" />
[|function [|{| "contextRangeIndex": 0 |}Bar|]() {
    // This is a reference to [|Bar|] in a comment.
    "this is a reference to [|Bar|] in a string";
    `Foo [|Bar|] Baz.`;
    {
        const Bar = 0;
        `[|Bar|] ba ${Bar} bara [|Bar|] berbobo ${Bar} araura [|Bar|] ara!`;
    }
}|]"#;
    let _s = Session::new_for_test("renameCommentsAndStrings4", content);
    // TODO: f.VerifyBaselineRename(t, nil /*preferences*/, f.Ranges()[1])
}
