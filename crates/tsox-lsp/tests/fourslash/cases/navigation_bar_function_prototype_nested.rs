use tsox_lsp::fourslash::Session;


#[test]
fn navigation_bar_function_prototype_nested() {
    let content = r#"// @allowJs: true
// @Filename: foo.js
function A() {}
A.B = function () {  } 
A.B.prototype.d = function () {  }  
Object.defineProperty(A.B.prototype, "x", {
    get() {}
})
A.prototype.D = function () {  } 
A.prototype.D.prototype.d = function () {  } "#;
    let _s = Session::new_for_test("navigationBarFunctionPrototypeNested", content);
    // TODO: f.VerifyBaselineDocumentSymbol(t)
}
