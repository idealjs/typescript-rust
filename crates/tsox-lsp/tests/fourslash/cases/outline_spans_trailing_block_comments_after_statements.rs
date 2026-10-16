use tsox_lsp::fourslash::Session;


#[test]
fn outline_spans_trailing_block_comments_after_statements() {
    let content = r#"console.log(0);
[|/*
/ * Some text
  */|]"#;
    let _s = Session::new_for_test("outlineSpansTrailingBlockCommentsAfterStatements", content);
    // TODO: f.VerifyOutliningSpans(t)
}
