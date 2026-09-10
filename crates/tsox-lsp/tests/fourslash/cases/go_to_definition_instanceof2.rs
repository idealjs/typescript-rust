use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineGoToDefinition"]
#[test]
fn go_to_definition_instanceof2() {
    let content = r#"// @lib: esnext
// @filename: /main.ts
class C {
  static /*end*/[Symbol.hasInstance](value: unknown): boolean { return true; }
}
declare var obj: any;
obj [|/*start*/instanceof|] C;"#;
    let mut s = Session::new_for_test("goToDefinitionInstanceof2", content);
    fourslash::unsupported("VerifyBaselineGoToDefinition"); // f.VerifyBaselineGoToDefinition(t, true, "start")
}
