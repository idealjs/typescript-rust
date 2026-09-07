use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineRename"]
#[test]
fn rename_unresolved_reexport1() {
    let content = r#"// @Filename: /a.ts
export { [|jsonSchema|] } from "@internal/ai-sdk-v4";
// @Filename: /b.ts
import { jsonSchema } from "./a";
"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineRename"); // f.VerifyBaselineRename(t, nil /*preferences*/, f.Ranges()[0])
}
