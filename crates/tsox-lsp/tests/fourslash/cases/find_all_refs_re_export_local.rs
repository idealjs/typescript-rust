use tsox_lsp::fourslash::{self, Session};


#[test]
fn find_all_refs_re_export_local() {
    let content = r#"// @noLib: true
// @strict: false
// @Filename: /a.ts
[|var /*ax0*/[|{| "isDefinition": true, "contextRangeIndex": 0 |}x|];|]
[|export { /*ax1*/[|{| "isWriteAccess": true, "isDefinition": true, "contextRangeIndex": 2 |}x|] };|]
[|export { /*ax2*/[|{| "contextRangeIndex": 4 |}x|] as /*ay*/[|{| "isWriteAccess": true, "isDefinition": true, "contextRangeIndex": 4 |}y|] };|]
// @Filename: /b.ts
[|import { /*bx0*/[|{| "isWriteAccess": true, "isDefinition": true, "contextRangeIndex": 7 |}x|], /*by0*/[|{| "isWriteAccess": true, "isDefinition": true, "contextRangeIndex": 7 |}y|] } from "./a";|]
/*bx1*/[|x|]; /*by1*/[|y|];"#;
    let mut s = Session::new_for_test("findAllRefsReExportLocal", content);
    fourslash::verify_no_errors(&mut s, );
    // TODO: f.VerifyBaselineFindAllReferences(t, "ax0", "ax1", "ax2", "bx0", "bx1", "ay", "by0", "by1")
    // TODO: f.VerifyBaselineRename(t, nil /*preferences*/, f.Ranges()[1], f.Ranges()[5])
    // TODO: f.VerifyBaselineRename(t, nil /*preferences*/, f.Ranges()[3])
    // TODO: f.VerifyBaselineRename(t, nil /*preferences*/, f.Ranges()[8], f.Ranges()[10])
    // TODO: f.VerifyBaselineRename(t, nil /*preferences*/, f.Ranges()[6])
    // TODO: f.VerifyBaselineRename(t, nil /*preferences*/, f.Ranges()[9], f.Ranges()[11])
}
