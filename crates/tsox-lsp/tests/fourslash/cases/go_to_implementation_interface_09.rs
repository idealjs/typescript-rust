use tsox_lsp::fourslash::Session;


#[test]
fn go_to_implementation_interface_09() {
    let content = r#"// @Filename: def.d.ts
export interface Interface { P: number }
// @Filename: ref.ts
import { Interface } from "./def";
const c: I/*ref*/nterface = [|{ P: 2 }|];"#;
    let _s = Session::new_for_test("goToImplementationInterface_09", content);
    // TODO: f.VerifyBaselineGoToImplementation(t, "ref")
}
