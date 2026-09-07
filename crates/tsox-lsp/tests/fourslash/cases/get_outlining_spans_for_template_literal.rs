use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: ${1}"]
#[test]
fn get_outlining_spans_for_template_literal() {
    let content = r#"declare function tag(...args: any[]): void
const a = [|` + "`" + `signal line` + "`" + `|]
const b = [|` + "`" + "#;
    // TODO: line` + "`" + `|]
    // TODO: line` + "`" + `|]
    // TODO: ${1}
    // TODO: line` + "`" + `|]
    // TODO: ${1}
    // TODO: line` + "`" + `|]
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyOutliningSpans"); // f.VerifyOutliningSpans(t)
}
