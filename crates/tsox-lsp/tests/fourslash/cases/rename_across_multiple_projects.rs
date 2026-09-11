use tsox_lsp::fourslash::{self, Session};


#[test]
fn rename_across_multiple_projects() {
    let content = r#"//@Filename: a.ts
[|var [|{| "contextRangeIndex": 0 |}x|]: number;|]
//@Filename: b.ts
/// <reference path="a.ts" />
[|x|]++;
//@Filename: c.ts
/// <reference path="a.ts" />
[|x|]++;"#;
    let mut s = Session::new_for_test("renameAcrossMultipleProjects", content);
    // TODO: f.VerifyBaselineRenameAtRangesWithText(t, nil /*preferences*/, "x")
}
