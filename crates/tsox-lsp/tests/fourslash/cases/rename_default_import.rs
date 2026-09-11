use tsox_lsp::fourslash::{self, Session};


#[test]
fn rename_default_import() {
    let content = r#"// @Filename: B.ts
[|export default class /*1*/[|{| "isWriteAccess": true, "isDefinition": true, "contextRangeIndex": 0 |}B|] {
    test() {
    }
}|]
// @Filename: A.ts
[|import /*2*/[|{| "isWriteAccess": true, "isDefinition": true, "contextRangeIndex": 2 |}B|] from "./B";|]
let b = new [|B|]();
b.test();"#;
    let mut s = Session::new_for_test("renameDefaultImport", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2")
    // TODO: f.VerifyBaselineRename(t, nil /*preferences*/, f.Ranges()[1], f.Ranges()[3], f.Ranges()[4])
    // TODO: f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, "1")
}
