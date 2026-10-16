use tsox_lsp::fourslash::Session;


#[test]
fn navigation_bar_function_prototype() {
    let content = r#"// @allowJs: true
// @Filename: foo.js
function f() {}
f.prototype.x = 0;
f.y = 0;
f.prototype.method = function () {};
Object.defineProperty(f, 'staticProp', { 
    set: function() {}, 
    get: function(){
    } 
});
Object.defineProperty(f.prototype, 'name', { 
    set: function() {}, 
    get: function(){
    } 
}); "#;
    let _s = Session::new_for_test("navigationBarFunctionPrototype", content);
    // TODO: f.VerifyBaselineDocumentSymbol(t)
}
