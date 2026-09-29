use tsox_lsp::fourslash::Session;


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
    let _s = Session::new_for_test("outliningHintSpansForFunction", content);
    // TODO: f.VerifyOutliningSpans(t)
}
