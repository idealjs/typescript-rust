use tsox_lsp::fourslash::Session;


#[test]
fn get_outlining_spans_for_template_literal() {
    let content = r#"declare function tag(...args: any[]): void
const a = [|`signal line`|]
const b = [|`multi
line`|]
const c = tag[|`signal line`|]
const d = tag[|`multi
line`|]
const e = [|`signal ${1} line`|]
const f = [|`multi
${1}
line`|]
const g = tag[|`signal ${1} line`|]
const h = tag[|`multi
${1}
line`|]
const i = ``"#;
    let _s = Session::new_for_test("getOutliningSpansForTemplateLiteral", content);
    // TODO: f.VerifyOutliningSpans(t)
}
