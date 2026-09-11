use tsox_lsp::fourslash::{self, Session};


#[test]
fn rename_alias() {
    let content = r#"namespace SomeModule { export class SomeClass { } }
[|import [|{| "contextRangeIndex": 0 |}M|] = SomeModule;|]
import C = [|M|].SomeClass;"#;
    let mut s = Session::new_for_test("renameAlias", content);
    // TODO: f.VerifyBaselineRenameAtRangesWithText(t, nil /*preferences*/, "M")
}
