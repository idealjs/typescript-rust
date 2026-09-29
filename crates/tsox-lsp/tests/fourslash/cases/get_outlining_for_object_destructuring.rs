use tsox_lsp::fourslash::Session;


#[test]
fn get_outlining_for_object_destructuring() {
    let content = r#"const[| {
    a,
    b,
    c
}|] =[| {
    a: 1,
    b: 2,
    c: 3
}|]
const[| {
    a:[| {
        a_1,
        a_2,
        a_3:[| {
            a_3_1,
            a_3_2,
            a_3_3,
        }|],
    }|],
    b,
    c
}|] =[| {
    a:[| {
        a_1: 1,
        a_2: 2,
        a_3:[| {
            a_3_1: 1,
            a_3_2: 1,
            a_3_3: 1
        }|],
    }|],
    b: 2,
    c: 3
}|]"#;
    let _s = Session::new_for_test("getOutliningForObjectDestructuring", content);
    // TODO: f.VerifyOutliningSpans(t)
}
