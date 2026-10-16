use tsox_lsp::fourslash::Session;


#[test]
fn go_to_definition_decorator_no_crash_on_function_declaration1() {
    let content = r#"function dec(target: any) { return target; }

@/*1*/dec
function foo() {}"#;
    let _s = Session::new_for_test("goToDefinitionDecoratorNoCrashOnFunctionDeclaration1", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "1")
}
