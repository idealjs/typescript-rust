use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineRename"]
#[test]
fn rename_alias_external_module2() {
    let content = r#"// @Filename: a.ts
[|module [|{| "contextRangeIndex": 0 |}SomeModule|] { export class SomeClass { } }|]
[|export = [|{| "contextRangeIndex": 2 |}SomeModule|];|]
// @Filename: b.ts
[|import [|{| "contextRangeIndex": 4 |}M|] = require("./a");|]
import C = [|M|].SomeClass;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineRename"); // f.VerifyBaselineRename(t, nil /*preferences*/, f.Ranges()[1], f.Ranges()[3], f.Ranges()[5], f.Ranges
}
