use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyOutliningSpans"]
#[test]
fn get_outlining_for_tuple_type() {
    let content = r#"type A =[| [
    number,
    number,
    number
]|]

type B =[| [
    [|[
        [|[
            number,
            number,
            number
        ]|]
    ]|]
]|]"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyOutliningSpans"); // f.VerifyOutliningSpans(t)
}
