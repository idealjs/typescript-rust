use tsox_lsp::fourslash::Session;


#[test]
fn navigation_bar_function_prototype3() {
    let content = r#"// @allowJs: true
// @Filename: foo.js
var A; 
A.prototype.a = function() { };
A.b = function() { };"#;
    let _s = Session::new_for_test("navigationBarFunctionPrototype3", content);
    // TODO: f.VerifyBaselineDocumentSymbol(t)
}
