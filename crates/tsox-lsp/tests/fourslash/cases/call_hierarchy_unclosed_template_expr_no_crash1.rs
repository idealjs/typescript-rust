use tsox_lsp::fourslash::{self, Session};


#[test]
fn call_hierarchy_unclosed_template_expr_no_crash1() {
    // TODO: // Regression test for a crash in prepareCallHierarchy caused by parser error
    // TODO: // recovery: when a template expression is truncated mid-call (e.g. `${format`
    // TODO: // subsequent HTML template literals as a TypeScript class declaration.
    // TODO: // The resulting anonymous ClassDeclaration (no name, no `default` modifier)
    // TODO: // previously caused a "Expected call hierarchy declaration to have a reference
    // TODO: // node" assertion failure.
    let content = r#"// @Filename: /main.ts\n"#;
    // TODO: "function updateBadge() {\n" +
    let mut s = Session::new_for_test("callHierarchyUnclosedTemplateExprNoCrash1", content);
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.VerifyBaselineCallHierarchy(t)
    // TODO: }
}
