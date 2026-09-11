use tsox_lsp::fourslash::{self, Session};


#[test]
fn find_all_refs_import_star_of_export_equals() {
    let content = r#"// @allowSyntheticDefaultimports: true
// @Filename: /node_modules/a/index.d.ts
[|declare function /*a0*/[|{| "isWriteAccess": true, "isDefinition": true, "contextRangeIndex": 0 |}a|](): void;|]
[|declare namespace /*a1*/[|{| "isWriteAccess": true, "isDefinition": true, "contextRangeIndex": 2 |}a|] {
    export const x: number;
}|]
[|export = /*a2*/[|{| "contextRangeIndex": 4 |}a|];|]
// @Filename: /b.ts
[|import /*b0*/[|{| "isWriteAccess": true, "isDefinition": true, "contextRangeIndex": 6 |}b|] from "a";|]
/*b1*/[|b|]();
[|b|].x;
// @Filename: /c.ts
[|import /*c0*/[|{| "isWriteAccess": true, "isDefinition": true, "contextRangeIndex": 10 |}a|] from "a";|]
/*c1*/[|a|]();
/*c2*/[|a|].x;"#;
    let mut s = Session::new_for_test("findAllRefsImportStarOfExportEquals", content);
    fourslash::verify_no_errors(&mut s, );
    // TODO: f.VerifyBaselineFindAllReferences(t, "a0", "a1", "a2", "b0", "b1", "c0", "c1", "c2")
    // TODO: f.VerifyBaselineRename(t, nil /*preferences*/, f.Ranges()[1], f.Ranges()[3], f.Ranges()[5])
    // TODO: f.VerifyBaselineRename(t, nil /*preferences*/, f.Ranges()[7], f.Ranges()[8], f.Ranges()[9])
    // TODO: f.VerifyBaselineRename(t, nil /*preferences*/, f.Ranges()[11], f.Ranges()[12], f.Ranges()[13])
}
