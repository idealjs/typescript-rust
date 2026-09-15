use tsox_lsp::fourslash::Session;


#[test]
fn outlining_spans_for_arrow_function_body() {
    let content = r#"() => 42;
() => ( 42 );
() =>[| {
    42
}|];
() => [|(
    42
)|];
() =>[| "foo" +
    "bar" +
    "baz"|];"#;
    let _s = Session::new_for_test("outliningSpansForArrowFunctionBody", content);
    // TODO: f.VerifyOutliningSpans(t)
}
