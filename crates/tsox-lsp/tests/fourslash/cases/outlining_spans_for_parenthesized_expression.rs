use tsox_lsp::fourslash::Session;


#[test]
fn outlining_spans_for_parenthesized_expression() {
    let content = r#"const a = [|(
    true
        ? true
        : false
            ? true
            : false
)|];

const b = ( 1 );

const c = [|(
    1
)|];

( 1 );

[|(
    [|(
        [|(
            1
        )|]
    )|]
)|];

[|(
    [|(
        ( 1 )
    )|]
)|];"#;
    let _s = Session::new_for_test("outliningSpansForParenthesizedExpression", content);
    // TODO: f.VerifyOutliningSpans(t)
}
