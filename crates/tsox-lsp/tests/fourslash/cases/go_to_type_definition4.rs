use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineGoToDefinition"]
#[test]
fn go_to_type_definition4() {
    let content = r#"// @Filename: foo.ts
export type /*def0*/T = string;
export const /*def1*/T = "";
// @Filename: bar.ts
import { T } from "./foo";
let x: [|/*reference*/T|];"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineGoToTypeDefinition"); // f.VerifyBaselineGoToTypeDefinition(t, "reference")
    fourslash::unsupported("VerifyBaselineGoToDefinition"); // f.VerifyBaselineGoToDefinition(t, true, "reference")
}
