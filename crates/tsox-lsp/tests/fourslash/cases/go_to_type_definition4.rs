use tsox_lsp::fourslash::Session;


#[test]
fn go_to_type_definition4() {
    let content = r#"// @Filename: foo.ts
export type /*def0*/T = string;
export const /*def1*/T = "";
// @Filename: bar.ts
import { T } from "./foo";
let x: [|/*reference*/T|];"#;
    let _s = Session::new_for_test("goToTypeDefinition4", content);
    // TODO: f.VerifyBaselineGoToTypeDefinition(t, "reference")
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "reference")
}
