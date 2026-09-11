use tsox_lsp::fourslash::{self, Session};


#[test]
fn go_to_definition_return7() {
    let content = r#"function foo(a: string, b: string): string;
function foo(a: number, b: number): number;
function /*end*/foo(a: any, b: any): any {
    [|/*start*/return|] a + b;
}"#;
    let mut s = Session::new_for_test("goToDefinitionReturn7", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "start")
}
