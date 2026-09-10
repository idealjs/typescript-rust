use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn find_all_references_dynamic_import3() {
    let content = r#"// @Filename: foo.ts
[|export function /*0*/[|{| "isWriteAccess": true, "isDefinition": true, "contextRangeIndex": 0 |}bar|]() { return "bar"; }|]
import('./foo').then(([|{ /*1*/[|{| "isWriteAccess": true, "isDefinition": true, "contextRangeIndex": 2 |}bar|] }|]) => undefined);"#;
    let mut s = Session::new_for_test("findAllReferencesDynamicImport3", content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "0", "1")
    fourslash::unsupported("VerifyBaselineRename"); // f.VerifyBaselineRename(t, nil /*preferences*/, f.Ranges()[1], f.Ranges()[3])
}
