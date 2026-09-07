use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn rename_import_of_export_equals() {
    let content = r#"[|declare namespace /*N*/[|{| "isWriteAccess": true, "isDefinition": true, "contextRangeIndex": 0 |}N|] {
    [|export var /*x*/[|{| "isWriteAccess": true, "isDefinition": true, "contextRangeIndex": 2 |}x|]: number;|]
}|]
declare module "mod" {
    [|export = [|{| "contextRangeIndex": 4 |}N|];|]
}
declare module "a" {
    [|import * as /*a*/[|{| "isWriteAccess": true, "isDefinition": true, "contextRangeIndex": 6 |}N|] from "mod";|]
    [|export { [|{| "isWriteAccess": true, "isDefinition": true, "contextRangeIndex": 8 |}N|] };|] // Renaming N here would rename
}
declare module "b" {
    [|import { /*b*/[|{| "isWriteAccess": true, "isDefinition": true, "contextRangeIndex": 10 |}N|] } from "a";|]
    export const y: typeof [|N|].[|x|];
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "N", "a", "b", "x")
    fourslash::unsupported("VerifyBaselineRename"); // f.VerifyBaselineRename(t, nil /*preferences*/, f.Ranges()[1], f.Ranges()[5])
    fourslash::unsupported("VerifyBaselineRename"); // f.VerifyBaselineRename(t, nil /*preferences*/, f.Ranges()[7])
    fourslash::unsupported("VerifyBaselineRename"); // f.VerifyBaselineRename(t, nil /*preferences*/, f.Ranges()[9])
    fourslash::unsupported("VerifyBaselineRename"); // f.VerifyBaselineRename(t, nil /*preferences*/, f.Ranges()[11], f.Ranges()[12])
    fourslash::unsupported("VerifyBaselineRename"); // f.VerifyBaselineRename(t, nil /*preferences*/, f.Ranges()[3], f.Ranges()[13])
}
