use tsox_lsp::fourslash::{self, Session};


#[test]
fn outline_spans_block_comments_without_statements() {
    let content = r#"[|/*
/ * Some text
  */|]"#;
    let mut s = Session::new_for_test("outlineSpansBlockCommentsWithoutStatements", content);
    // TODO: f.VerifyOutliningSpans(t)
}
