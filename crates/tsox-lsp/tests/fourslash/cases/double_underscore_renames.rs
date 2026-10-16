use tsox_lsp::fourslash::Session;


#[test]
fn double_underscore_renames() {
    let content = r#"// @Filename: fileA.ts
[|export function [|{| "contextRangeIndex": 0 |}__foo|]() {
}|]

// @Filename: fileB.ts
[|import { [|{| "contextRangeIndex": 2 |}__foo|] as bar } from "./fileA";|]

bar();"#;
    let _s = Session::new_for_test("doubleUnderscoreRenames", content);
    // TODO: f.VerifyBaselineRenameAtRangesWithText(t, nil /*preferences*/, "__foo")
}
