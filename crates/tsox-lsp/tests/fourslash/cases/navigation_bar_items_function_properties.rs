use tsox_lsp::fourslash::Session;


#[test]
fn navigation_bar_items_function_properties() {
    let content = r#"(function(){
var A;
A/*1*/
.a = function() { };
})();"#;
    let _s = Session::new_for_test("navigationBarItemsFunctionProperties", content);
    // TODO: f.VerifyBaselineDocumentSymbol(t)
}
