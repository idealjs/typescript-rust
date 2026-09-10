use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCodeFixNotAvailable"]
#[test]
fn convert_function_to_es6_class_no_quick_info_for_iife() {
    let content = r#"// @allowJs: true
// @Filename: /a.js
(/*1*/function () {
   const foo = () => {
        this.x = 10;
   };
   foo;
})();"#;
    let mut s = Session::new_for_test("convertFunctionToEs6Class_noQuickInfoForIIFE", content);
    fourslash::go_to_marker(&mut s, "1");
    fourslash::unsupported("VerifyCodeFixNotAvailable"); // f.VerifyCodeFixNotAvailable(t)
}
