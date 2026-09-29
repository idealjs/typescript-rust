use tsox_lsp::fourslash::Session;


#[test]
fn rename_unresolved_reexport1() {
    let content = r#"// @Filename: /a.ts
export { [|jsonSchema|] } from "@internal/ai-sdk-v4";
// @Filename: /b.ts
import { jsonSchema } from "./a";
"#;
    let _s = Session::new_for_test("renameUnresolvedReexport1", content);
    // TODO: f.VerifyBaselineRename(t, nil /*preferences*/, f.Ranges()[0])
}
