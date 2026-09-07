use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineRenameAtRangesWithText"]
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
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineRenameAtRangesWithText"); // f.VerifyBaselineRenameAtRangesWithText(t, nil /*preferences*/, "x")
}
