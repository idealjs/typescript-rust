use tsox_lsp::fourslash::{self, Session};


#[test]
fn rename_import_of_re_export() {
    let content = r#"// @noLib: true
declare module "a" {
    [|export class /*1*/[|{| "isWriteAccess": true, "isDefinition": true, "contextRangeIndex": 0 |}C|] {}|]
}
declare module "b" {
    [|export { /*2*/[|{| "isWriteAccess": true, "isDefinition": true, "contextRangeIndex": 2 |}C|] } from "a";|]
}
declare module "c" {
    [|import { /*3*/[|{| "isWriteAccess": true, "isDefinition": true, "contextRangeIndex": 4 |}C|] } from "b";|]
    export function f(c: [|C|]): void;
}"#;
    let mut s = Session::new_for_test("renameImportOfReExport", content);
    fourslash::verify_no_errors(&mut s, );
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2", "3")
    // TODO: f.VerifyBaselineRename(t, nil /*preferences*/, f.Ranges()[1])
    // TODO: f.VerifyBaselineRename(t, nil /*preferences*/, f.Ranges()[3])
    // TODO: f.VerifyBaselineRename(t, nil /*preferences*/, f.Ranges()[5], f.Ranges()[6])
}
