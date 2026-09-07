use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineRenameAtRangesWithText"]
#[test]
fn double_underscore_renames() {
    let content = r#"// @Filename: fileA.ts
[|export function [|{| "contextRangeIndex": 0 |}__foo|]() {
}|]

// @Filename: fileB.ts
[|import { [|{| "contextRangeIndex": 2 |}__foo|] as bar } from "./fileA";|]

bar();"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineRenameAtRangesWithText"); // f.VerifyBaselineRenameAtRangesWithText(t, nil /*preferences*/, "__foo")
}
