use tsox_lsp::fourslash::{self, Session};


#[test]
fn rename_import_of_export_equals2() {
    let content = r#"[|declare namespace /*N*/[|{| "isWriteAccess": true, "isDefinition": true, "contextRangeIndex": 0 |}N|] {
    export var x: number;
}|]
declare module "mod" {
    [|export = [|{| "contextRangeIndex": 2 |}N|];|]
}
declare module "a" {
    [|import * as /*O*/[|{| "isWriteAccess": true, "isDefinition": true, "contextRangeIndex": 4 |}O|] from "mod";|]
    [|export { [|{| "contextRangeIndex": 6 |}O|] as /*P*/[|{| "isWriteAccess": true, "isDefinition": true, "contextRangeIndex": 6 |}P|] };|] // Renaming N here would rename
}
declare module "b" {
    [|import { [|{| "contextRangeIndex": 9 |}P|] as /*Q*/[|{| "isWriteAccess": true, "isDefinition": true, "contextRangeIndex": 9 |}Q|] } from "a";|]
    export const y: typeof [|Q|].x;
}"#;
    let mut s = Session::new_for_test("renameImportOfExportEquals2", content);
    fourslash::verify_no_errors(&mut s, );
    // TODO: f.VerifyBaselineFindAllReferences(t, "N", "O", "P", "Q")
    // TODO: f.VerifyBaselineRenameAtRangesWithText(t, nil /*preferences*/, "N", "O", "P", "Q")
}
