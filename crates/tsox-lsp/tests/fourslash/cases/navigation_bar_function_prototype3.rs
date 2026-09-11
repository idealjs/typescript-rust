use tsox_lsp::fourslash::{self, Session};


#[test]
fn navigation_bar_function_prototype3() {
    let content = r#"// @allowJs: true
// @Filename: foo.js
var A; 
A.prototype.a = function() { };
A.b = function() { };"#;
    let mut s = Session::new_for_test("navigationBarFunctionPrototype3", content);
    // TODO: f.VerifyBaselineDocumentSymbol(t)
}
