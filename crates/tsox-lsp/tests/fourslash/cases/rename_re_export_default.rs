use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineRename"]
#[test]
fn rename_re_export_default() {
    let content = r#"// @Filename: /a.ts
export { default } from "./b";
[|export { default as [|{| "contextRangeIndex": 0 |}b|] } from "./b";|]
export { default as bee } from "./b";
[|import { default as [|{| "contextRangeIndex": 2 |}b|] } from "./b";|]
import { default as bee } from "./b";
[|import [|{| "contextRangeIndex": 4 |}b|] from "./b";|]
// @Filename: /b.ts
[|const [|{| "contextRangeIndex": 6 |}b|] = 0;|]
[|export default [|{| "contextRangeIndex": 8 |}b|];|]"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineRename"); // f.VerifyBaselineRename(t, nil /*preferences*/, f.Ranges()[1], f.Ranges()[3], f.Ranges()[5], f.Ranges
}
