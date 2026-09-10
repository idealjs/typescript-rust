use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineDocumentSymbol"]
#[test]
fn navigation_bar_function_prototype2() {
    let content = r#"// @allowJs: true
// @Filename: foo.js
A.prototype.a = function() { };
A.prototype.b = function() { };
function A() {}"#;
    let mut s = Session::new_for_test("navigationBarFunctionPrototype2", content);
    fourslash::unsupported("VerifyBaselineDocumentSymbol"); // f.VerifyBaselineDocumentSymbol(t)
}
