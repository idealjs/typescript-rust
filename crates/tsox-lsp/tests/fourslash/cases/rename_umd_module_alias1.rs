use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineRenameAtRangesWithText"]
#[test]
fn rename_umd_module_alias1() {
    let content = r#"// @Filename: 0.d.ts
export function doThing(): string;
export function doTheOtherThing(): void;
[|export as namespace [|{| "contextRangeIndex": 0 |}myLib|];|]
// @Filename: 1.ts
/// <reference path="0.d.ts" />
[|myLib|].doThing();"#;
    let mut s = Session::new_for_test("renameUMDModuleAlias1", content);
    fourslash::unsupported("VerifyBaselineRenameAtRangesWithText"); // f.VerifyBaselineRenameAtRangesWithText(t, nil /*preferences*/, "myLib")
}
