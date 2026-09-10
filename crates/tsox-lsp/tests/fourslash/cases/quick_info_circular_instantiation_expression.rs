use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineHover"]
#[test]
fn quick_info_circular_instantiation_expression() {
    let content = r#"declare function foo<T>(t: T): typeof foo<T>;
/**/foo("");"#;
    let mut s = Session::new_for_test("quickInfoCircularInstantiationExpression", content);
    fourslash::unsupported("VerifyBaselineHover"); // f.VerifyBaselineHover(t)
}
