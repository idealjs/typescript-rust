use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineRenameAtRangesWithText"]
#[test]
fn rename_alias_external_module() {
    let content = r#"// @Filename: a.ts
namespace SomeModule { export class SomeClass { } }
export = SomeModule;
// @Filename: b.ts
[|import [|{| "contextRangeIndex": 0 |}M|] = require("./a");|]
import C = [|M|].SomeClass;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineRenameAtRangesWithText"); // f.VerifyBaselineRenameAtRangesWithText(t, nil /*preferences*/, "M")
}
