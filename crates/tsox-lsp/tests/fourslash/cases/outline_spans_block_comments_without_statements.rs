use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyOutliningSpans"]
#[test]
fn outline_spans_block_comments_without_statements() {
    let content = r#"[|/*
/ * Some text
  */|]"#;
    let mut s = Session::new_for_test("outlineSpansBlockCommentsWithoutStatements", content);
    fourslash::unsupported("VerifyOutliningSpans"); // f.VerifyOutliningSpans(t)
}
