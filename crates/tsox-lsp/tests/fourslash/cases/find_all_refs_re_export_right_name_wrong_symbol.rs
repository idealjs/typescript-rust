use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn find_all_refs_re_export_right_name_wrong_symbol() {
    let content = r#"// @Filename: /a.ts
[|export const /*a*/[|{| "isWriteAccess": true, "isDefinition": true, "contextRangeIndex": 0 |}x|] = 0;|]
// @Filename: /b.ts
[|export const /*b*/[|{| "isWriteAccess": true, "isDefinition": true, "contextRangeIndex": 2 |}x|] = 0;|]
//@Filename: /c.ts
[|export { /*cFromB*/[|{| "isWriteAccess": true, "isDefinition": true, "contextRangeIndex": 4 |}x|] } from "./b";|]
[|import { /*cFromA*/[|{| "isWriteAccess": true, "isDefinition": true, "contextRangeIndex": 6 |}x|] } from "./a";|]
/*cUse*/[|x|];
// @Filename: /d.ts
[|import { /*d*/[|{| "isWriteAccess": true, "isDefinition": true, "contextRangeIndex": 9 |}x|] } from "./c";|]"#;
    let mut s = Session::new_for_test("findAllRefsReExportRightNameWrongSymbol", content);
    fourslash::verify_no_errors(&mut s, );
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "a", "b", "cFromB", "cFromA", "cUse", "d")
    fourslash::unsupported("VerifyBaselineRename"); // f.VerifyBaselineRename(t, nil /*preferences*/, f.Ranges()[1])
    fourslash::unsupported("VerifyBaselineRename"); // f.VerifyBaselineRename(t, nil /*preferences*/, f.Ranges()[7], f.Ranges()[8])
    fourslash::unsupported("VerifyBaselineRename"); // f.VerifyBaselineRename(t, nil /*preferences*/, f.Ranges()[3])
    fourslash::unsupported("VerifyBaselineRename"); // f.VerifyBaselineRename(t, nil /*preferences*/, f.Ranges()[5])
    fourslash::unsupported("VerifyBaselineRename"); // f.VerifyBaselineRename(t, nil /*preferences*/, f.Ranges()[10])
}
