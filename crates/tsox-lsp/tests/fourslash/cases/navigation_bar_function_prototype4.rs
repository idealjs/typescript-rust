use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineDocumentSymbol"]
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
    let mut s = Session::new_for_test("navigationBarFunctionPrototype4", content);
    fourslash::unsupported("VerifyBaselineDocumentSymbol"); // f.VerifyBaselineDocumentSymbol(t)
}
