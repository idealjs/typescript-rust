use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyOutliningSpans"]
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
    let mut s = Session::new_for_test("outliningSpansForParenthesizedExpression", content);
    fourslash::unsupported("VerifyOutliningSpans"); // f.VerifyOutliningSpans(t)
}
