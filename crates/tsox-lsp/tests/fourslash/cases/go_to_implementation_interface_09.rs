use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineGoToImplementation"]
#[test]
fn go_to_implementation_interface_09() {
    let content = r#"// @Filename: def.d.ts
export interface Interface { P: number }
// @Filename: ref.ts
import { Interface } from "./def";
const c: I/*ref*/nterface = [|{ P: 2 }|];"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineGoToImplementation"); // f.VerifyBaselineGoToImplementation(t, "ref")
}
