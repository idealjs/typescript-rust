use tsox_lsp::fourslash::Session;


#[test]
fn rename_alias3() {
    let content = r#"namespace SomeModule { [|export class [|{| "contextRangeIndex": 0 |}SomeClass|] { }|] }
import M = SomeModule;
import C = M.[|SomeClass|];"#;
    let _s = Session::new_for_test("renameAlias3", content);
    // TODO: f.VerifyBaselineRenameAtRangesWithText(t, nil /*preferences*/, "SomeClass")
}
