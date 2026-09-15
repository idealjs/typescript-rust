use tsox_lsp::fourslash::Session;


#[test]
fn quick_info_circular_instantiation_expression() {
    let content = r#"declare function foo<T>(t: T): typeof foo<T>;
/**/foo("");"#;
    let _s = Session::new_for_test("quickInfoCircularInstantiationExpression", content);
    // TODO: f.VerifyBaselineHover(t)
}
