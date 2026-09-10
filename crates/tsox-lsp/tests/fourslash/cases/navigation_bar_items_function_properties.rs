use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineDocumentSymbol"]
#[test]
fn navigation_bar_items_function_properties() {
    let content = r#"(function(){
var A;
A/*1*/
.a = function() { };
})();"#;
    let mut s = Session::new_for_test("navigationBarItemsFunctionProperties", content);
    fourslash::unsupported("VerifyBaselineDocumentSymbol"); // f.VerifyBaselineDocumentSymbol(t)
}
