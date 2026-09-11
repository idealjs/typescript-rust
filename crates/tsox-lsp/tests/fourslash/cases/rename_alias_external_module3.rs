use tsox_lsp::fourslash::{self, Session};


#[test]
fn rename_alias_external_module3() {
    let content = r#"// @Filename: a.ts
namespace SomeModule { [|export class [|{| "contextRangeIndex": 0 |}SomeClass|] { }|] }
export = SomeModule;
// @Filename: b.ts
import M = require("./a");
import C = M.[|SomeClass|];"#;
    let mut s = Session::new_for_test("renameAliasExternalModule3", content);
    // TODO: f.VerifyBaselineRenameAtRangesWithText(t, nil /*preferences*/, "SomeClass")
}
