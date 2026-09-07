use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyOutliningSpans"]
#[test]
fn outlining_hint_spans_for_function() {
    let content = r#"namespace NS[| {
    function f(x: number, y: number)[| {
        return x + y;
    }|]

    function g[|(
        x: number,
        y: number,
    ): number {
        return x + y;
    }|]
}|]"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyOutliningSpans"); // f.VerifyOutliningSpans(t)
}
