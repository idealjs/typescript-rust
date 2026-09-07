use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn find_all_refs_prefix_suffix_preference() {
    let content = r#"// @Filename: /file1.ts
declare function log(s: string | number): void;
[|const /*q0*/[|{| "isWriteAccess": true, "isDefinition": true, "contextRangeIndex": 0 |}q|] = 1;|]
[|export { /*q1*/[|{| "isWriteAccess": true, "isDefinition": true, "contextRangeIndex": 2 |}q|] };|]
const x = {
    [|/*z0*/[|{| "isWriteAccess": true, "isDefinition": true, "contextRangeIndex": 4 |}z|]: 'value'|]
}
[|const { /*z1*/[|{| "isWriteAccess": true, "isDefinition": true, "contextRangeIndex": 6 |}z|] } = x;|]
log(/*z2*/[|z|]);
// @Filename: /file2.ts
declare function log(s: string | number): void;
[|import { /*q2*/[|{| "isWriteAccess": true, "isDefinition": true, "contextRangeIndex": 9 |}q|] } from "./file1";|]
log(/*q3*/[|q|] + 1);"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyNoErrors"); // f.VerifyNoErrors(t)
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "q0", "q1", "q2", "q3", "z0", "z1", "z2")
    fourslash::unsupported("VerifyBaselineRename"); // f.VerifyBaselineRename(t, &lsutil.UserPreferences{UseAliasesForRename: core.TSTrue}, f.Ranges()[1], 
    fourslash::unsupported("VerifyBaselineRename"); // f.VerifyBaselineRename(t, &lsutil.UserPreferences{UseAliasesForRename: core.TSFalse}, f.Ranges()[1],
    fourslash::unsupported("VerifyBaselineRename"); // f.VerifyBaselineRename(t, &lsutil.UserPreferences{UseAliasesForRename: core.TSTrue}, f.Ranges()[5], 
    fourslash::unsupported("VerifyBaselineRename"); // f.VerifyBaselineRename(t, &lsutil.UserPreferences{UseAliasesForRename: core.TSFalse}, f.Ranges()[5],
}
