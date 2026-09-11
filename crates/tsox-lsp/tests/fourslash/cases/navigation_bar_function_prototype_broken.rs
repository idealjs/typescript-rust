use tsox_lsp::fourslash::{self, Session};


#[test]
fn navigation_bar_function_prototype_broken() {
    let content = r#"// @allowJs: true
// @Filename: foo.js
function A() {}
A. // Started typing something here
A.prototype.a = function() { };
G. // Started typing something here
A.prototype.a = function() { };"#;
    let mut s = Session::new_for_test("navigationBarFunctionPrototypeBroken", content);
    // TODO: f.VerifyBaselineDocumentSymbol(t)
}
