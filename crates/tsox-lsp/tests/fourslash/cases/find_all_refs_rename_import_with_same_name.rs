use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn find_all_refs_rename_import_with_same_name() {
    let content = r#"// @Filename: /a.ts
[|export const /*0*/[|{| "isWriteAccess": true, "isDefinition": true, "contextRangeIndex": 0 |}x|] = 0;|]
//@Filename: /b.ts
[|import { /*1*/[|{| "contextRangeIndex": 2 |}x|] as /*2*/[|{| "isWriteAccess": true, "isDefinition": true, "contextRangeIndex": 2 |}x|] } from "./a";|]
/*3*/[|x|];"#;
    let mut s = Session::new_for_test("findAllRefsRenameImportWithSameName", content);
    fourslash::verify_no_errors(&mut s, );
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "0", "1", "2", "3")
    fourslash::unsupported("VerifyBaselineRename"); // f.VerifyBaselineRename(t, nil /*preferences*/, f.Ranges()[1], f.Ranges()[3], f.Ranges()[4], f.Ranges
}
