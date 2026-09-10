use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineDocumentSymbol"]
#[test]
fn navigation_bar_function_prototype_interlaced() {
    let content = r#"// @allowJs: true
// @Filename: foo.js
var b = 1;
function A() {}; 
A.prototype.a = function() { };
A.b = function() { };
b = 2
/* Comment */
A.prototype.c = function() { }
var b = 2
A.prototype.d = function() { }"#;
    let mut s = Session::new_for_test("navigationBarFunctionPrototypeInterlaced", content);
    fourslash::unsupported("VerifyBaselineDocumentSymbol"); // f.VerifyBaselineDocumentSymbol(t)
}
