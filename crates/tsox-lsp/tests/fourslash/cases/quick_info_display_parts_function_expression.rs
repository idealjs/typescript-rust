use tsox_lsp::fourslash::{self, Session};


#[test]
fn quick_info_display_parts_function_expression() {
    let content = r#"var /*1*/x = function /*2*/foo() {
    /*3*/foo();
};
var /*4*/y = function () {
};
(function /*5*/foo1() {
    /*6*/foo1();
})();"#;
    let mut s = Session::new_for_test("quickInfoDisplayPartsFunctionExpression", content);
    // TODO: f.VerifyBaselineHover(t)
}
