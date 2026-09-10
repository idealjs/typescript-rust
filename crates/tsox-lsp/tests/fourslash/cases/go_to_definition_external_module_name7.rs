use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineGoToDefinition"]
#[test]
fn go_to_definition_external_module_name7() {
    let content = r#"// @Filename: b.ts
import {Foo, Bar} from [|'e/*1*/'|];
// @Filename: a.ts
declare module /*2*/"e" {
    class Foo { }
}"#;
    let mut s = Session::new_for_test("goToDefinitionExternalModuleName7", content);
    fourslash::unsupported("VerifyBaselineGoToDefinition"); // f.VerifyBaselineGoToDefinition(t, true, "1")
}
