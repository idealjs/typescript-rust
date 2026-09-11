use tsox_lsp::fourslash::{self, Session};


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
    let mut s = Session::new_for_test("getOutliningForTupleType", content);
    // TODO: f.VerifyOutliningSpans(t)
}
