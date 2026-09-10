use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineDocumentSymbol"]
#[test]
fn navigation_bar_function_prototype3() {
    let content = r#"// @allowJs: true
// @Filename: foo.js
var A; 
A.prototype.a = function() { };
A.b = function() { };"#;
    let mut s = Session::new_for_test("navigationBarFunctionPrototype3", content);
    fourslash::unsupported("VerifyBaselineDocumentSymbol"); // f.VerifyBaselineDocumentSymbol(t)
}
