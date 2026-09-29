use tsox_lsp::fourslash::{self, Session};


#[test]
fn transitive_export_imports2() {
    let content = r#"// @Filename: a.ts
[|namespace /*A*/[|{| "isWriteAccess": true, "isDefinition": true, "contextRangeIndex": 0 |}A|] {
    export const x = 0;
}|]
// @Filename: b.ts
[|export import /*B*/[|{| "isWriteAccess": true, "isDefinition": true, "contextRangeIndex": 2 |}B|] = [|A|];|]
[|B|].x;
// @Filename: c.ts
[|import { /*C*/[|{| "isWriteAccess": true, "isDefinition": true, "contextRangeIndex": 6 |}B|] } from "./b";|]"#;
    let mut s = Session::new_for_test("transitiveExportImports2", content);
    fourslash::verify_no_errors(&mut s, );
    // TODO: f.VerifyBaselineFindAllReferences(t, "A", "B", "C")
    // TODO: f.VerifyBaselineRename(t, nil /*preferences*/, f.Ranges()[1], f.Ranges()[4])
    // TODO: f.VerifyBaselineRename(t, nil /*preferences*/, f.Ranges()[3], f.Ranges()[5])
    // TODO: f.VerifyBaselineRename(t, nil /*preferences*/, f.Ranges()[7])
}
