use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineGoToDefinition"]
#[test]
fn tsx_go_to_definition_class_in_different_file() {
    let content = r#"// @jsx: preserve
// @Filename: C.tsx
export default class /*def*/C {}
// @Filename: a.tsx
import C from "./C";
const foo = </*use*/C />;"#;
    let mut s = Session::new_for_test("tsxGoToDefinitionClassInDifferentFile", content);
    fourslash::verify_no_errors(&mut s, );
    fourslash::unsupported("VerifyBaselineGoToDefinition"); // f.VerifyBaselineGoToDefinition(t, false, "use")
}
