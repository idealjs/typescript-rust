use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineRename"]
#[test]
fn rename_import_require() {
    let content = r#"// @Filename: /a.ts
[|import [|{| "contextRangeIndex": 0 |}e|] = require("mod4");|]
[|e|];
a = { [|e|] };
[|export { [|{| "contextRangeIndex": 4 |}e|] };|]
// @Filename: /b.ts
[|import { [|{| "contextRangeIndex": 6 |}e|] } from "./a";|]
[|export { [|{| "contextRangeIndex": 8 |}e|] };|]"#;
    let mut s = Session::new_for_test("renameImportRequire", content);
    fourslash::unsupported("VerifyBaselineRename"); // f.VerifyBaselineRename(t, nil /*preferences*/, f.Ranges()[1], f.Ranges()[2], f.Ranges()[3], f.Ranges
}
