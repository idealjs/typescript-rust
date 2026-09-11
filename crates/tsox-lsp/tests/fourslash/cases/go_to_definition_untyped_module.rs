use tsox_lsp::fourslash::{self, Session};


#[test]
fn go_to_definition_untyped_module() {
    let content = r#"// @Filename: /node_modules/foo/index.js
not read
// @Filename: /a.ts
import { /*def*/f } from "foo";
[|/*use*/f|]();"#;
    let mut s = Session::new_for_test("goToDefinition_untypedModule", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "use")
}
