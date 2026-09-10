use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineRenameAtRangesWithText"]
#[test]
fn rename_alias3() {
    let content = r#"namespace SomeModule { [|export class [|{| "contextRangeIndex": 0 |}SomeClass|] { }|] }
import M = SomeModule;
import C = M.[|SomeClass|];"#;
    let mut s = Session::new_for_test("renameAlias3", content);
    fourslash::unsupported("VerifyBaselineRenameAtRangesWithText"); // f.VerifyBaselineRenameAtRangesWithText(t, nil /*preferences*/, "SomeClass")
}
