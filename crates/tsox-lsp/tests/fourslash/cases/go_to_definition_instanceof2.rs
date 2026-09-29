use tsox_lsp::fourslash::Session;


#[test]
fn go_to_definition_instanceof2() {
    let content = r#"// @lib: esnext
// @filename: /main.ts
class C {
  static /*end*/[Symbol.hasInstance](value: unknown): boolean { return true; }
}
declare var obj: any;
obj [|/*start*/instanceof|] C;"#;
    let _s = Session::new_for_test("goToDefinitionInstanceof2", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "start")
}
