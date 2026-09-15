use tsox_lsp::fourslash::Session;


#[test]
fn get_outlining_for_array_destructuring() {
    let content = r#"const[| [
    a,
    b,
    c
]|] =[| [
    1,
    2,
    3
]|];
const[| [
    [|[
        [|[
            [|[
                a,
                b,
                c
            ]|]
        ]|]
    ]|],
    [|[
        a1,
        b1,
        c1
    ]|]
]|] =[| [
    [|[
        [|[
            [|[
                1,
                2,
                3
            ]|]
        ]|]
    ]|],
    [|[
        1,
        2,
        3
    ]|]
]|]"#;
    let _s = Session::new_for_test("getOutliningForArrayDestructuring", content);
    // TODO: f.VerifyOutliningSpans(t)
}
