use tsox_lsp::fourslash::Session;


#[test]
fn navigation_bar_function_prototype4() {
    let content = r#"// @allowJs: true
// @Filename: foo.js
var A; 
A.prototype = { };
A.prototype = { m() {} };
A.prototype.a = function() { };
A.b = function() { };

var B; 
B["prototype"] = { };
B["prototype"] = { m() {} };
B["prototype"]["a"] = function() { };
B["b"] = function() { };"#;
    let _s = Session::new_for_test("navigationBarFunctionPrototype4", content);
    // TODO: f.VerifyBaselineDocumentSymbol(t)
}
