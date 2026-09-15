use tsox_lsp::fourslash::Session;


#[test]
fn rename_alias2() {
    let content = r#"[|module [|{| "contextRangeIndex": 0 |}SomeModule|] { export class SomeClass { } }|]
import M = [|SomeModule|];
import C = M.SomeClass;"#;
    let _s = Session::new_for_test("renameAlias2", content);
    // TODO: f.VerifyBaselineRenameAtRangesWithText(t, nil /*preferences*/, "SomeModule")
}
