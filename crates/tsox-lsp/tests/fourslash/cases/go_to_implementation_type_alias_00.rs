use tsox_lsp::fourslash::{self, Session};


#[test]
fn go_to_implementation_type_alias_00() {
    let content = r#"// @Filename: def.d.ts
export type TypeAlias = { P: number }
// @Filename: ref.ts
import { TypeAlias } from "./def";
const c: T/*ref*/ypeAlias = [|{ P: 2 }|];"#;
    let mut s = Session::new_for_test("goToImplementationTypeAlias_00", content);
    // TODO: f.VerifyBaselineGoToImplementation(t, "ref")
}
