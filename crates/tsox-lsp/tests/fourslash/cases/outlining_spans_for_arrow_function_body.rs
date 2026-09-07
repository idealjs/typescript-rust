use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyOutliningSpans"]
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
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyOutliningSpans"); // f.VerifyOutliningSpans(t)
}
