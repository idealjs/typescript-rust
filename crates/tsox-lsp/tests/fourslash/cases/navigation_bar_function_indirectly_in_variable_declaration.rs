use tsox_lsp::fourslash::{self, Session};


#[test]
fn navigation_bar_function_indirectly_in_variable_declaration() {
    let content = r#"var a = {
    propA: function() {
        var c;
    }
};
var b;
b = {
    propB: function() {
    // function must not have an empty body to appear top level
        var d;
    }
};"#;
    let mut s = Session::new_for_test("navigationBarFunctionIndirectlyInVariableDeclaration", content);
    // TODO: f.VerifyBaselineDocumentSymbol(t)
}
