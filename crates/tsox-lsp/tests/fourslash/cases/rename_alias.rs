use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineRenameAtRangesWithText"]
#[test]
fn rename_alias() {
    let content = r#"namespace SomeModule { export class SomeClass { } }
[|import [|{| "contextRangeIndex": 0 |}M|] = SomeModule;|]
import C = [|M|].SomeClass;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineRenameAtRangesWithText"); // f.VerifyBaselineRenameAtRangesWithText(t, nil /*preferences*/, "M")
}
