use tsox_lsp::fourslash::{self, Session};


#[test]
fn find_all_refs_on_import_aliases2() {
    let content = r#"//@Filename: a.ts
[|export class /*class0*/[|{| "isWriteAccess": true, "isDefinition": true, "contextRangeIndex": 0 |}Class|] {}|]
//@Filename: b.ts
[|import { /*class1*/[|{| "contextRangeIndex": 2 |}Class|] as /*c2_0*/[|{| "isWriteAccess": true, "isDefinition": true, "contextRangeIndex": 2 |}C2|] } from "./a";|]
var c = new /*c2_1*/[|C2|]();
//@Filename: c.ts
[|export { /*class2*/[|{| "contextRangeIndex": 6 |}Class|] as /*c3*/[|{| "isWriteAccess": true, "isDefinition": true, "contextRangeIndex": 6 |}C3|] } from "./a";|]"#;
    let mut s = Session::new_for_test("findAllRefsOnImportAliases2", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "class0", "class1", "class2", "c2_0", "c2_1", "c3")
    // TODO: f.VerifyBaselineRenameAtRangesWithText(t, nil /*preferences*/, "Class", "C2", "C3")
}
