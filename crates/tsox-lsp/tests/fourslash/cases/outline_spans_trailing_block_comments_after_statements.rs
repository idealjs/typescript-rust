use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyOutliningSpans"]
#[test]
fn outline_spans_trailing_block_comments_after_statements() {
    let content = r#"console.log(0);
[|/*
/ * Some text
  */|]"#;
    let mut s = Session::new_for_test("outlineSpansTrailingBlockCommentsAfterStatements", content);
    fourslash::unsupported("VerifyOutliningSpans"); // f.VerifyOutliningSpans(t)
}
