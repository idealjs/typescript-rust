use tsox_lsp::fourslash::{self, Session};


#[test]
fn navigation_bar_anonymous_class_and_function_expressions2() {
    let content = r#"console.log(console.log(class Y {}, class X {}), console.log(class B {}, class A {}));
console.log(class Cls { meth() {} });"#;
    let mut s = Session::new_for_test("navigationBarAnonymousClassAndFunctionExpressions2", content);
    // TODO: f.VerifyBaselineDocumentSymbol(t)
}
