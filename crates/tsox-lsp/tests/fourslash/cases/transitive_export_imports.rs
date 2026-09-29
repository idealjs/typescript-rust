use tsox_lsp::fourslash::{self, Session};


#[test]
fn transitive_export_imports() {
    let content = r#"// @module: commonjs
// @Filename: a.ts
[|class /*1*/[|{| "isWriteAccess": true, "isDefinition": true, "contextRangeIndex": 0 |}A|] {
}|]
[|export = [|{| "contextRangeIndex": 2 |}A|];|]
// @Filename: b.ts
[|export import /*2*/[|{| "isWriteAccess": true, "isDefinition": true, "contextRangeIndex": 4 |}b|] = require('./a');|]
// @Filename: c.ts
[|import /*3*/[|{| "isWriteAccess": true, "isDefinition": true, "contextRangeIndex": 6 |}b|] = require('./b');|]
var a = new /*4*/[|b|]./**/[|b|]();"#;
    let mut s = Session::new_for_test("transitiveExportImports", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyQuickInfoExists(t)
    fourslash::verify_no_errors(&mut s, );
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2", "3", "4")
    // TODO: f.VerifyBaselineRename(t, nil /*preferences*/, f.Ranges()[1], f.Ranges()[3])
    // TODO: f.VerifyBaselineRename(t, nil /*preferences*/, f.Ranges()[5], f.Ranges()[9])
    // TODO: f.VerifyBaselineRename(t, nil /*preferences*/, f.Ranges()[7], f.Ranges()[8])
}
