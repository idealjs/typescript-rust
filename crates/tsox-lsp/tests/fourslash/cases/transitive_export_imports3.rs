use tsox_lsp::fourslash::{self, Session};


#[test]
fn transitive_export_imports3() {
    let content = r#"// @Filename: a.ts
[|export function /*f*/[|{| "isWriteAccess": true, "isDefinition": true, "contextRangeIndex": 0 |}f|]() {}|]
// @Filename: b.ts
[|export { [|{| "contextRangeIndex": 2 |}f|] as /*g0*/[|{| "isWriteAccess": true, "isDefinition": true, "contextRangeIndex": 2 |}g|] } from "./a";|]
[|import { /*f2*/[|{| "isWriteAccess": true, "isDefinition": true, "contextRangeIndex": 5 |}f|] } from "./a";|]
[|import { /*g1*/[|{| "isWriteAccess": true, "isDefinition": true, "contextRangeIndex": 7 |}g|] } from "./b";|]"#;
    let mut s = Session::new_for_test("transitiveExportImports3", content);
    fourslash::verify_no_errors(&mut s, );
    // TODO: f.VerifyBaselineFindAllReferences(t, "f", "g0", "g1", "f2")
    // TODO: f.VerifyBaselineRename(t, nil /*preferences*/, f.Ranges()[1], f.Ranges()[3])
    // TODO: f.VerifyBaselineRename(t, nil /*preferences*/, f.Ranges()[6])
    // TODO: f.VerifyBaselineRename(t, nil /*preferences*/, f.Ranges()[4])
    // TODO: f.VerifyBaselineRename(t, nil /*preferences*/, f.Ranges()[8])
}
