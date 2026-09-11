use tsox_lsp::fourslash::{self, Session};


#[test]
fn rename01() {
    let content = r#"// @lib: es5
///<reference path="./Bar.ts" />
[|function [|{| "contextRangeIndex": 0 |}Bar|]() {
    // This is a reference to [|Bar|] in a comment.
    "this is a reference to [|Bar|] in a string"
}|]"#;
    let mut s = Session::new_for_test("rename01", content);
    // TODO: f.MarkTestAsStradaServer()
    // TODO: f.VerifyBaselineRename(t, nil /*preferences*/, f.Ranges()[1])
}
