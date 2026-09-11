use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn completion_list_in_named_function_expression() {
    let content = r#"function foo(a: number): string {
    /*insideFunctionDeclaration*/
    return "";
}

(function foo(): number {
    /*insideFunctionExpression*/
    fo/*referenceInsideFunctionExpression*/o;
    return "";
})

/*globalScope*/
fo/*referenceInGlobalScope*/o;"#;
    let mut s = Session::new_for_test("completionListInNamedFunctionExpression", content);
    // TODO: f.VerifyCompletions(t, []string{"globalScope", "insideFunctionDeclaration", "insideFunctionExpressio
    fourslash::verify_quick_info_at(&mut s, "referenceInsideFunctionExpression", "(local function) foo(): number", "");
    fourslash::verify_quick_info_at(&mut s, "referenceInGlobalScope", "function foo(a: number): string", "");
}
