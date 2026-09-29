use tsox_lsp::fourslash::Session;


#[test]
fn navigation_bar_function_prototype2() {
    let content = r#"// @allowJs: true
// @Filename: foo.js
A.prototype.a = function() { };
A.prototype.b = function() { };
function A() {}"#;
    let _s = Session::new_for_test("navigationBarFunctionPrototype2", content);
    // TODO: f.VerifyBaselineDocumentSymbol(t)
}
