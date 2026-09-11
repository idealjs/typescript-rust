use tsox_lsp::fourslash::{self, Session};


#[test]
fn rename_import_of_re_export2() {
    let content = r#"declare module "a" {
    [|export class /*1*/[|{| "isWriteAccess": true, "isDefinition": true, "contextRangeIndex": 0 |}C|] {}|]
}
declare module "b" {
    [|export { [|{| "contextRangeIndex": 2 |}C|] as /*2*/[|{| "isWriteAccess": true, "isDefinition": true, "contextRangeIndex": 2 |}D|] } from "a";|]
}
declare module "c" {
    [|import { /*3*/[|{| "isWriteAccess": true, "isDefinition": true, "contextRangeIndex": 5 |}D|] } from "b";|]
    export function f(c: [|D|]): void;
}"#;
    let mut s = Session::new_for_test("renameImportOfReExport2", content);
    fourslash::verify_no_errors(&mut s, );
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2", "3")
    // TODO: f.VerifyBaselineRename(t, nil /*preferences*/, ToAny(f.GetRangesByText().Get("C"))...)
    // TODO: f.VerifyBaselineRename(t, nil /*preferences*/, f.GetRangesByText().Get("D")[0])
    // TODO: f.VerifyBaselineRename(t, nil /*preferences*/, f.GetRangesByText().Get("D")[1], f.GetRangesByText().
}
